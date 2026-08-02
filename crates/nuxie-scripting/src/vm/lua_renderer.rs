// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer.cpp
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;

use luaur_rt::{AnyUserData, Error, Result, Table, UserData, UserDataMethods, Value};
use nuxie_render_api::{Factory as RenderFactory, Renderer};

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
        let value: Value = table.get("draw")?;
        let Value::Function(function) = value else {
            return Ok(());
        };

        let lua = table.lua();
        self.verify_render_context(factory)?;
        let save_count = Rc::new(Cell::new(0usize));
        let valid = Rc::new(Cell::new(true));
        let renderer_ref = Rc::new(RefCell::new(erase_renderer_lifetime(renderer)));

        let scripted_renderer = lua.create_userdata(ScriptedRenderer {
            renderer: Rc::clone(&renderer_ref),
            bindings: self.clone(),
            save_count: Rc::clone(&save_count),
            valid: Rc::clone(&valid),
        })?;
        let result = function.call::<()>((table.clone(), scripted_renderer));

        while save_count.get() > 0 {
            let mut renderer = renderer_ref.borrow_mut();
            // The renderer userdata is still valid while this cleanup runs;
            // the pointer is invalidated immediately after the save stack is
            // balanced.
            unsafe { renderer.as_mut().restore() };
            save_count.set(save_count.get() - 1);
        }
        valid.set(false);
        result
    }
}

fn erase_renderer_lifetime(renderer: &mut dyn Renderer) -> NonNull<dyn Renderer> {
    let ptr: NonNull<dyn Renderer + '_> = NonNull::from(renderer);
    // The pointer is held only by userdata created for one draw call; `valid`
    // is cleared before `call_draw` returns.
    unsafe { mem::transmute::<NonNull<dyn Renderer + '_>, NonNull<dyn Renderer>>(ptr) }
}

pub(super) struct ScriptedRenderer {
    renderer: Rc<RefCell<NonNull<dyn Renderer>>>,
    pub(super) bindings: RendererBindings,
    save_count: Rc<Cell<usize>>,
    valid: Rc<Cell<bool>>,
}

impl ScriptedRenderer {
    fn validate(&self) -> Result<()> {
        if self.valid.get() {
            Ok(())
        } else {
            Err(Error::runtime("Renderer is no longer valid"))
        }
    }

    pub(super) fn renderer_mut(&self) -> Result<std::cell::RefMut<'_, NonNull<dyn Renderer>>> {
        self.validate()?;
        Ok(self.renderer.borrow_mut())
    }

    fn save(&self) -> Result<()> {
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().save() };
        self.save_count.set(self.save_count.get() + 1);
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.save_count.get() == 0 {
            return Err(Error::runtime("Renderer save/restore stack was unbalanced"));
        }
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().restore() };
        self.save_count.set(self.save_count.get() - 1);
        Ok(())
    }
}

impl UserData for ScriptedRenderer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("save", |_, this, ()| this.save());
        methods.add_method("restore", |_, this, ()| this.restore());
        methods.add_method("transform", |_, this, matrix: AnyUserData| {
            this.validate()?;
            let matrix = matrix.borrow::<ScriptedMat2D>()?;
            let mut renderer = this.renderer_mut()?;
            unsafe { renderer.as_mut().transform(matrix.0) };
            Ok(())
        });
        methods.add_method("clipPath", |_, this, path: AnyUserData| {
            this.validate()?;
            let mut path = path.borrow_mut::<ScriptedPath>()?;
            this.bindings.with_factory(|factory| {
                let render_path = path.render_path(factory);
                let mut renderer = this.renderer_mut()?;
                unsafe { renderer.as_mut().clip_path(render_path) };
                Ok(())
            })
        });
        methods.add_method(
            "drawPath",
            |_, this, (path, paint): (AnyUserData, AnyUserData)| {
                this.validate()?;
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                let paint = paint.borrow::<ScriptedPaint>()?;
                this.bindings.with_factory(|factory| {
                    let render_path = path.render_path(factory);
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer
                            .as_mut()
                            .draw_path(render_path, paint.render_paint.as_ref())
                    };
                    Ok(())
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
                this.validate()?;
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let blend_mode = parse_blend_mode_name(&blend_mode)?;
                with_scripted_image(&image, |image| {
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer.as_mut().draw_image(
                            Some(image),
                            sampler.0,
                            blend_mode,
                            opacity,
                        )
                    };
                    Ok(())
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
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer.as_mut().draw_image_mesh(
                            Some(image),
                            sampler.0,
                            vertices.render_buffer(),
                            uvs.render_buffer(),
                            indices.render_buffer(),
                            vertex_count,
                            index_count,
                            blend_mode,
                            opacity,
                        )
                    };
                    Ok(())
                })?
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::ScriptVm;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{NoopScriptHost, ScriptInstance};

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
