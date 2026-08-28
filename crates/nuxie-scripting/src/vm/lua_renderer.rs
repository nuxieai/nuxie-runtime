// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer.cpp
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;

use luaur_rt::{AnyUserData, Error, Result, Table, UserData, UserDataMethods, Value};
use nuxie_render_api::{Factory as RenderFactory, RenderCanvasFrame, Renderer};

use super::lua_image::{ScriptedImage, ScriptedImageSampler};
use super::lua_mat2d::ScriptedMat2D;
use super::lua_mesh::{ScriptedTriangleBuffer, ScriptedVertexBuffer};
use super::lua_paint::{ScriptedPaint, parse_blend_mode_name};
use super::lua_path::ScriptedPath;
use super::lua_renderer_library::RendererBindings;
use crate::gpu_canvas::{GpuCanvasImage, with_gpu_canvas_image};

fn with_scripted_image<R>(
    image: &AnyUserData,
    callback: impl FnOnce(&dyn nuxie_render_api::RenderImage) -> R,
) -> Result<R> {
    if image.is::<ScriptedImage>() {
        return image.borrow::<ScriptedImage>()?.with_render_image(callback);
    }
    if image.is::<GpuCanvasImage>() {
        let image = image.borrow::<GpuCanvasImage>()?;
        return with_gpu_canvas_image(&image, callback);
    }
    Err(Error::runtime("expected Image userdata"))
}

impl RendererBindings {
    pub(crate) fn call_draw(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<()> {
        self.call_draw_with_balance(table, factory, renderer)
            .map(|_| ())
    }

    pub(super) fn call_draw_with_balance(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<bool> {
        let lua = table.lua();
        self.verify_render_context(factory)?;
        let (scripted_renderer, _renderer_scope) =
            ScriptedRenderer::create_call_scoped_userdata(&lua, renderer, self.clone())?;
        let field: Value = table.get("draw")?;
        let result = match field {
            Value::Function(function) => {
                function.call::<()>((table.clone(), scripted_renderer.clone()))
            }
            // Legacy files advertise every optional method. C++ treats a
            // currently missing or non-function draw field as a balanced
            // no-op after installing the renderer userdata.
            _ => Ok(()),
        };

        let balanced = {
            let scripted_renderer = scripted_renderer.borrow::<ScriptedRenderer>()?;
            scripted_renderer.end()
        };
        result.map(|_| balanced)
    }
}

struct ActiveRenderer {
    token: u64,
    renderer: NonNull<dyn Renderer>,
}

thread_local! {
    static ACTIVE_RENDERERS: RefCell<Vec<ActiveRenderer>> = const { RefCell::new(Vec::new()) };
    static NEXT_RENDERER_TOKEN: Cell<u64> = const { Cell::new(1) };
}

pub(super) struct ScopedRendererAccess<'a> {
    token: u64,
    _exclusive_borrow: PhantomData<&'a mut dyn Renderer>,
}

impl ScopedRendererAccess<'_> {
    fn new(renderer: &mut dyn Renderer) -> Self {
        let token = NEXT_RENDERER_TOKEN.with(|next| {
            let token = next.get();
            next.set(token.wrapping_add(1).max(1));
            token
        });
        let ptr: NonNull<dyn Renderer + '_> = NonNull::from(renderer);
        // SAFETY: the erased pointer is stored only in the thread-local active
        // call stack. This guard carries the exclusive borrow and removes the
        // pointer before that borrow can end; Lua userdata retains only `token`.
        let renderer =
            unsafe { mem::transmute::<NonNull<dyn Renderer + '_>, NonNull<dyn Renderer>>(ptr) };
        ACTIVE_RENDERERS.with(|active| {
            active.borrow_mut().push(ActiveRenderer { token, renderer });
        });
        Self {
            token,
            _exclusive_borrow: PhantomData,
        }
    }
}

impl Drop for ScopedRendererAccess<'_> {
    fn drop(&mut self) {
        ACTIVE_RENDERERS.with(|active| {
            let mut active = active.borrow_mut();
            let frame = active.pop().expect("scoped renderer stack underflow");
            assert_eq!(frame.token, self.token, "scoped renderer stack order");
        });
    }
}

fn with_active_renderer<R>(
    token: u64,
    callback: impl FnOnce(&mut dyn Renderer) -> Result<R>,
) -> Result<R> {
    ACTIVE_RENDERERS.with(|active| {
        let mut active = active
            .try_borrow_mut()
            .map_err(|_| Error::runtime("Renderer is already mutably borrowed by this callback"))?;
        let frame = active
            .iter_mut()
            .rev()
            .find(|frame| frame.token == token)
            .ok_or_else(|| Error::lua_l_runtime("Renderer is no longer valid."))?;
        // SAFETY: ScopedRendererAccess owns the exclusive borrow for the
        // entire time this frame is present. The RefCell borrow above makes
        // every access through this token unique, including reentrant Lua.
        callback(unsafe { frame.renderer.as_mut() })
    })
}

enum RendererTarget {
    CallScoped(u64),
    CanvasFrame(Rc<RefCell<Box<dyn RenderCanvasFrame>>>),
}

pub(super) struct ScriptedRenderer {
    target: RefCell<Option<RendererTarget>>,
    pub(super) bindings: RendererBindings,
    save_count: Cell<usize>,
}

impl ScriptedRenderer {
    pub(super) fn create_call_scoped_userdata<'a>(
        lua: &luaur_rt::Lua,
        renderer: &'a mut dyn Renderer,
        bindings: RendererBindings,
    ) -> Result<(AnyUserData, ScopedRendererAccess<'a>)> {
        let access = ScopedRendererAccess::new(renderer);
        let userdata = lua.create_userdata(Self {
            target: RefCell::new(Some(RendererTarget::CallScoped(access.token))),
            bindings,
            save_count: Cell::new(0),
        })?;
        Ok((userdata, access))
    }

    pub(super) fn create_canvas_userdata(
        lua: &luaur_rt::Lua,
        frame: Rc<RefCell<Box<dyn RenderCanvasFrame>>>,
        bindings: RendererBindings,
    ) -> Result<AnyUserData> {
        lua.create_userdata(Self {
            target: RefCell::new(Some(RendererTarget::CanvasFrame(frame))),
            bindings,
            save_count: Cell::new(0),
        })
    }

    pub(super) fn end(&self) -> bool {
        let balanced = self.save_count.get() == 0;
        while self.save_count.get() > 0 {
            if self
                .with_renderer_mut(|renderer| {
                    renderer.restore();
                    Ok(())
                })
                .is_err()
            {
                break;
            }
            self.save_count.set(self.save_count.get() - 1);
        }
        self.target.borrow_mut().take();
        balanced
    }

    pub(super) fn with_renderer_mut<R>(
        &self,
        callback: impl FnOnce(&mut dyn Renderer) -> Result<R>,
    ) -> Result<R> {
        let target = self.target.borrow();
        match target
            .as_ref()
            .ok_or_else(|| Error::lua_l_runtime("Renderer is no longer valid."))?
        {
            RendererTarget::CallScoped(token) => with_active_renderer(*token, callback),
            RendererTarget::CanvasFrame(frame) => {
                let mut frame = frame.try_borrow_mut().map_err(|_| {
                    Error::runtime("Renderer is already mutably borrowed by this callback")
                })?;
                callback(frame.renderer())
            }
        }
    }

    fn save(&self) -> Result<()> {
        self.with_renderer_mut(|renderer| {
            renderer.save();
            Ok(())
        })?;
        self.save_count.set(self.save_count.get() + 1);
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.save_count.get() == 0 {
            return Err(Error::runtime("Renderer save/restore stack was unbalanced"));
        }
        self.with_renderer_mut(|renderer| {
            renderer.restore();
            Ok(())
        })?;
        self.save_count.set(self.save_count.get() - 1);
        Ok(())
    }
}

impl UserData for ScriptedRenderer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("save", |_, this, ()| this.save());
        methods.add_method("restore", |_, this, ()| this.restore());
        methods.add_method("transform", |_, this, matrix: AnyUserData| {
            let matrix = matrix.borrow::<ScriptedMat2D>()?;
            this.with_renderer_mut(|renderer| {
                renderer.transform(matrix.0);
                Ok(())
            })
        });
        methods.add_method("clipPath", |_, this, path: AnyUserData| {
            let mut path = path.borrow_mut::<ScriptedPath>()?;
            this.bindings.with_factory(|factory| {
                let render_path = path.render_path(factory);
                this.with_renderer_mut(|renderer| {
                    renderer.clip_path(render_path);
                    Ok(())
                })
            })
        });
        methods.add_method(
            "drawPath",
            |_, this, (path, paint): (AnyUserData, AnyUserData)| {
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                let paint = paint.borrow::<ScriptedPaint>()?;
                this.bindings.with_factory(|factory| {
                    let render_path = path.render_path(factory);
                    this.with_renderer_mut(|renderer| {
                        renderer.draw_path(render_path, paint.render_paint.as_ref());
                        Ok(())
                    })
                })
            },
        );
        methods.add_method(
            "drawImage",
            |_,
             this,
             (image, sampler, blend_mode, opacity): (
                AnyUserData,
                AnyUserData,
                String,
                f32,
            )| {
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let blend_mode = parse_blend_mode_name(&blend_mode)?;
                with_scripted_image(&image, |image| {
                    this.with_renderer_mut(|renderer| {
                        renderer.draw_image(
                            Some(image),
                            sampler.0,
                            blend_mode,
                            opacity,
                        );
                        Ok(())
                    })
                })?
            },
        );
        methods.add_method(
            "drawImageMesh",
            |_,
             this,
             (image, sampler, vertices, uvs, indices, blend_mode, opacity): (
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                String,
                f32,
            )| {
                this.validate()?;
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let blend_mode = parse_blend_mode_name(&blend_mode)?;

                let vertex_count = vertices.borrow::<ScriptedVertexBuffer>()?.len();
                let uv_count = uvs.borrow::<ScriptedVertexBuffer>()?.len();
                let index_count = indices.borrow::<ScriptedTriangleBuffer>()?.len();
                indices
                    .borrow::<ScriptedTriangleBuffer>()?
                    .validate_for_vertices(vertex_count, uv_count)?;
                let vertex_count = u32::try_from(vertex_count)
                    .map_err(|_| Error::runtime("vertex count exceeds u32"))?;
                let index_count = u32::try_from(index_count)
                    .map_err(|_| Error::runtime("index count exceeds u32"))?;

                this.bindings.with_factory(|factory| {
                    vertices
                        .borrow_mut::<ScriptedVertexBuffer>()?
                        .update(factory);
                    uvs.borrow_mut::<ScriptedVertexBuffer>()?.update(factory);
                    indices
                        .borrow_mut::<ScriptedTriangleBuffer>()?
                        .update(factory);
                    Ok(())
                })?;

                let vertices = vertices.borrow::<ScriptedVertexBuffer>()?;
                let uvs = uvs.borrow::<ScriptedVertexBuffer>()?;
                let indices = indices.borrow::<ScriptedTriangleBuffer>()?;
                with_scripted_image(&image, |image| {
                    this.with_renderer_mut(|renderer| {
                        renderer.draw_image_mesh(
                            Some(image),
                            sampler.0,
                            vertices.render_buffer(),
                            uvs.render_buffer(),
                            indices.render_buffer(),
                            vertex_count,
                            index_count,
                            blend_mode,
                            opacity,
                        );
                        Ok(())
                    })
                })?
            },
        );
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use crate::vm::ScriptVm;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{NoopScriptHost, ScriptInstance};

    #[test]
    fn scripted_renderer_keeps_callback_lifetime_state_inline() {
        fn assert_field_type<T: 'static>(
            _field: impl Fn(&ScriptedRenderer) -> &T,
            expected: std::any::TypeId,
        ) {
            assert_eq!(std::any::TypeId::of::<T>(), expected);
        }

        assert_field_type(
            |renderer| &renderer.target,
            std::any::TypeId::of::<RefCell<Option<RendererTarget>>>(),
        );
        assert_field_type(
            |renderer| &renderer.save_count,
            std::any::TypeId::of::<Cell<usize>>(),
        );
    }

    #[test]
    fn retained_scripted_renderer_is_invalid_after_balanced_callback_cleanup() {
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let table: Table = vm
            .eval(
                r#"
                return {
                    draw = function(self, renderer)
                        renderer:save()
                        retainedRenderer = renderer
                    end,
                }
                "#,
            )
            .unwrap();
        let mut instance = vm.script_instance_from_table(table);
        let mut renderer = factory.borrow().make_renderer();

        instance
            .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
            .unwrap();

        assert!(factory.borrow().stream().contains("save\nrestore\n"));
        let (valid, error): (bool, String) = vm
            .eval(
                "local ok, err = pcall(function() retainedRenderer:save() end); \
                 return ok, tostring(err)",
            )
            .unwrap();
        assert!(!valid);
        assert!(error.contains("Renderer is no longer valid"), "{error}");
    }

    #[test]
    fn scripted_renderer_end_reports_an_unbalanced_save_stack() {
        const SOURCE: &str = "function render(renderer:Renderer):()\n\
  local path:Path = Path.new()\n\
  local paint:Paint = Paint.new()\n\
  renderer:save()\n\
  renderer:drawPath(path, paint)\n\
  renderer:save()\n\
end\n";

        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        vm.lua()
            .load(SOURCE)
            .set_name("test_source")
            .exec()
            .unwrap();
        let table: Table = vm
            .eval("return { draw = function(self, renderer) return render(renderer) end }")
            .unwrap();
        let mut renderer = factory.borrow().make_renderer();

        let balanced = vm
            .renderer_bindings
            .call_draw_with_balance(&table, &mut factory, &mut renderer)
            .unwrap();

        assert!(!balanced);
        let stream = factory.borrow().stream();
        let renderer_actions: Vec<_> = stream
            .lines()
            .filter_map(|line| match line {
                "save" | "restore" => Some(line),
                line if line.starts_with("drawPath ") => Some("drawPath"),
                _ => None,
            })
            .collect();
        assert_eq!(
            renderer_actions,
            ["save", "drawPath", "save", "restore", "restore"]
        );
    }

    #[test]
    fn factory_image_mesh_allocates_resets_and_draws_exact_buffers() {
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let image = factory
            .borrow_mut()
            .decode_image(&[1, 2, 3, 4])
            .expect("recording image");
        vm.lua()
            .globals()
            .set(
                "image",
                vm.lua()
                    .create_userdata(ScriptedImage::from_render_image(image))
                    .unwrap(),
            )
            .unwrap();

        let table: Table = vm
            .eval(
                r#"
                return {
                    draw = function(self, renderer)
                        local vertices = VertexBuffer()
                        vertices:add(Vector.xy(99, 99))
                        vertices:reset()
                        vertices:add(Vector.xy(0, 0), Vector.xy(10, 0), Vector.xy(10, 10))
                        local uvs = VertexBuffer()
                        uvs:add(Vector.xy(0, 0), Vector.xy(1, 0), Vector.xy(1, 1))
                        local indices = TriangleBuffer()
                        indices:add(2, 1, 0)
                        renderer:drawImageMesh(
                            image,
                            ImageSampler("repeat", "mirror", "nearest"),
                            vertices,
                            uvs,
                            indices,
                            "multiply",
                            0.25)
                    end,
                }
                "#,
            )
            .unwrap();
        let mut instance = vm.script_instance_from_table(table);
        let mut renderer = factory.borrow().make_renderer();
        instance
            .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
            .unwrap();

        let stream = factory.borrow().stream();
        assert!(
            stream.contains("makeRenderBuffer id=1 type=1 flags=1 size=24"),
            "{stream}"
        );
        assert!(
            stream.contains("makeRenderBuffer id=2 type=1 flags=1 size=24"),
            "{stream}"
        );
        assert!(
            stream.contains("makeRenderBuffer id=3 type=0 flags=1 size=6"),
            "{stream}"
        );
        assert!(
            stream.contains("drawImageMesh image=1 sampler={wrapX=1,wrapY=2,filter=1,key=16} vertices=1 uvs=2 indices=3 vertexCount=3 indexCount=3 blendMode=24 opacity=0.25"),
            "{stream}"
        );
    }

    #[test]
    fn image_mesh_rejects_out_of_bounds_triangles_before_factory_allocation() {
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let image = factory.borrow_mut().decode_image(&[1]).unwrap();
        vm.lua()
            .globals()
            .set(
                "image",
                vm.lua()
                    .create_userdata(ScriptedImage::from_render_image(image))
                    .unwrap(),
            )
            .unwrap();
        let table: Table = vm
            .eval(
                r#"
                return { draw = function(self, renderer)
                    local vertices = VertexBuffer()
                    vertices:add(Vector.xy(0, 0))
                    local uvs = VertexBuffer()
                    uvs:add(Vector.xy(0, 0))
                    local indices = TriangleBuffer()
                    indices:add(0, 1, 0)
                    renderer:drawImageMesh(image, ImageSampler("clamp", "clamp", "bilinear"), vertices, uvs, indices, "srcOver", 1)
                end }
                "#,
            )
            .unwrap();
        let mut instance = vm.script_instance_from_table(table);
        let mut renderer = factory.borrow().make_renderer();
        let error = instance
            .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("triangle index 1 exceeds vertex buffer bounds 1"),
            "{error}"
        );
        assert!(!factory.borrow().stream().contains("makeRenderBuffer"));
    }
}
