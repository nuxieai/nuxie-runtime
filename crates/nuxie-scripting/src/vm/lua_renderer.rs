// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer.cpp
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::luau::{
    AnyUserData, Error, Loan, Result, ScriptLoan, Table, UserData, UserDataMethods, Value,
    reborrow_loan,
};
use nuxie_render_api::{Factory as RenderFactory, RenderCanvasFrame, Renderer};

use super::lua_image::{ScriptedImage, ScriptedImageSampler};
use super::lua_mat2d::ScriptedMat2D;
use super::lua_mesh::{ScriptedImageMeshInstances, ScriptedTriangleBuffer, ScriptedVertexBuffer};
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
        owner: Option<super::view_model::ScriptedPropertyListenerOwner>,
    ) -> Result<()> {
        self.call_draw_with_balance(table, factory, renderer, owner)
            .map(|_| ())
    }

    pub(super) fn call_draw_with_balance(
        &self,
        table: &Table,
        _factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        owner: Option<super::view_model::ScriptedPropertyListenerOwner>,
    ) -> Result<bool> {
        let lua = table.lua();
        // An e949 artboard instance may use a different factory facade. Its
        // borrowed renderer is independent of the VM's retained allocation
        // factory, which remains installed for script-created resources.
        let (scripted_renderer, mut scope) =
            ScriptedRenderer::create_call_scoped_userdata(&lua, renderer, self.clone())?;
        let field: Value = table.get_field("draw")?;
        let result = match field {
            Value::Function(function) => {
                let _property_owner = owner.map(|owner| {
                    super::view_model::ScriptViewModelFrameContext::for_lua(&lua).enter_owner(owner)
                });
                super::ProtectedScriptCall::protected_call_with::<()>(
                    &function,
                    Some(&mut scope),
                    (table.clone(), scripted_renderer.clone()),
                )
            }
            // Legacy files advertise every optional method. C++ treats a
            // currently missing or non-function draw field as a balanced
            // no-op after installing the renderer userdata.
            _ => Ok(()),
        };

        let balanced = {
            let scripted_renderer = scripted_renderer.borrow::<ScriptedRenderer>()?;
            scripted_renderer.end(Some(&mut scope))
        };
        result.map(|_| balanced)
    }
}

/// A renderer scope: the renderer one script call may draw into, lent to
/// that call ([`ScriptLoan`]). The script reaches it through the
/// `Renderer` userdata whose target names this scope's token; the loan
/// answers only to that token, and only while the call it was lent to
/// runs.
pub(super) struct RendererScope<'a> {
    token: u64,
    renderer: &'a mut dyn Renderer,
}

/// Renderer scope tokens: unique for the process, so a userdata kept from
/// one scope never names another.
static NEXT_RENDERER_TOKEN: AtomicU64 = AtomicU64::new(1);

impl<'a> RendererScope<'a> {
    pub(super) fn new(renderer: &'a mut dyn Renderer) -> Self {
        Self {
            token: NEXT_RENDERER_TOKEN.fetch_add(1, Ordering::Relaxed),
            renderer,
        }
    }
}

impl ScriptLoan for RendererScope<'_> {
    fn renderer(&mut self, token: u64) -> Option<&mut dyn Renderer> {
        if token == self.token {
            Some(&mut *self.renderer)
        } else {
            None
        }
    }
}

/// Marks a userdata's renderer as held by a host call until dropped.
struct InUse<'a>(&'a Cell<bool>);

impl<'a> InUse<'a> {
    fn enter(flag: &'a Cell<bool>) -> Self {
        flag.set(true);
        Self(flag)
    }
}

impl Drop for InUse<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

#[derive(Clone)]
enum RendererTarget {
    CallScoped(u64),
    CanvasFrame(Rc<RefCell<Box<dyn RenderCanvasFrame>>>),
}

pub(super) struct ScriptedRenderer {
    target: RefCell<Option<RendererTarget>>,
    pub(super) bindings: RendererBindings,
    save_count: Cell<usize>,
    /// Set while a host call holds this userdata's renderer: Lua run under
    /// that call (other than through a scope reborrowed from it) cannot
    /// reach the same renderer.
    in_use: Cell<bool>,
}

fn already_borrowed() -> Error {
    Error::runtime("Renderer is already mutably borrowed by this callback")
}

fn no_longer_valid() -> Error {
    Error::lua_l_runtime("Renderer is no longer valid.")
}

impl ScriptedRenderer {
    /// A `Renderer` userdata for a new scope over `renderer`, and that
    /// scope, to be lent to the call the userdata is passed to.
    pub(super) fn create_call_scoped_userdata<'a>(
        lua: &crate::luau::Lua,
        renderer: &'a mut dyn Renderer,
        bindings: RendererBindings,
    ) -> Result<(AnyUserData, RendererScope<'a>)> {
        let scope = RendererScope::new(renderer);
        let userdata = lua.create_userdata(Self {
            target: RefCell::new(Some(RendererTarget::CallScoped(scope.token))),
            bindings,
            save_count: Cell::new(0),
            in_use: Cell::new(false),
        })?;
        Ok((userdata, scope))
    }

    pub(super) fn create_canvas_userdata(
        lua: &crate::luau::Lua,
        frame: Rc<RefCell<Box<dyn RenderCanvasFrame>>>,
        bindings: RendererBindings,
    ) -> Result<AnyUserData> {
        lua.create_userdata(Self {
            target: RefCell::new(Some(RendererTarget::CanvasFrame(frame))),
            bindings,
            save_count: Cell::new(0),
            in_use: Cell::new(false),
        })
    }

    /// Restores what the script saved and detaches the userdata from its
    /// target; whether the saves were balanced. `loan` is the scope's,
    /// for a call-scoped target.
    pub(super) fn end(&self, mut loan: Loan<'_>) -> bool {
        let balanced = self.save_count.get() == 0;
        while self.save_count.get() > 0 {
            if self
                .with_renderer_mut(reborrow_loan(&mut loan), |renderer| {
                    renderer.restore();
                    Ok(())
                })
                .is_err()
            {
                break;
            }
            self.save_count.set(self.save_count.get().saturating_sub(1));
        }
        if let Ok(mut target) = self.target.try_borrow_mut() {
            *target = None;
        }
        balanced
    }

    /// Runs `callback` with the renderer this userdata targets: the
    /// canvas frame's, or, for a call-scoped target, the one `loan` lends
    /// to its scope.
    pub(super) fn with_renderer_mut<R>(
        &self,
        loan: Loan<'_>,
        callback: impl FnOnce(&mut dyn Renderer) -> Result<R>,
    ) -> Result<R> {
        let target = self
            .target
            .try_borrow()
            .map_err(|_| already_borrowed())?
            .clone();
        match target.ok_or_else(no_longer_valid)? {
            RendererTarget::CallScoped(token) => {
                if self.in_use.get() {
                    return Err(already_borrowed());
                }
                let renderer = loan
                    .and_then(|loan| loan.renderer(token))
                    .ok_or_else(no_longer_valid)?;
                let _in_use = InUse::enter(&self.in_use);
                callback(renderer)
            }
            RendererTarget::CanvasFrame(frame) => {
                let mut frame = frame.try_borrow_mut().map_err(|_| already_borrowed())?;
                callback(frame.renderer())
            }
        }
    }

    /// Runs `callback` with a new scope over `renderer` (reborrowed from
    /// the renderer this userdata's host call holds), this userdata
    /// targeting it meanwhile: Lua run with that scope lent draws through
    /// the same userdata into `renderer`.
    pub(super) fn with_reborrowed_renderer<R>(
        &self,
        renderer: &mut dyn Renderer,
        callback: impl FnOnce(&mut RendererScope<'_>) -> Result<R>,
    ) -> Result<R> {
        let mut scope = RendererScope::new(renderer);
        let mut target = self.target.try_borrow_mut().map_err(|_| already_borrowed())?;
        let previous = target.replace(RendererTarget::CallScoped(scope.token));
        drop(target);
        let _restore = RestoreTarget {
            renderer: self,
            previous,
            in_use: self.in_use.replace(false),
        };
        callback(&mut scope)
    }

    pub(super) fn save_count(&self) -> usize {
        self.save_count.get()
    }

    pub(super) fn restore_to(&self, mut loan: Loan<'_>, count: usize) -> Result<()> {
        while self.save_count.get() > count {
            self.restore(reborrow_loan(&mut loan))?;
        }
        Ok(())
    }

    fn save(&self, loan: Loan<'_>) -> Result<()> {
        self.with_renderer_mut(loan, |renderer| {
            renderer.save();
            Ok(())
        })?;
        self.save_count.set(self.save_count.get().saturating_add(1));
        Ok(())
    }

    fn restore(&self, loan: Loan<'_>) -> Result<()> {
        if self.save_count.get() == 0 {
            return Err(Error::runtime("Renderer save/restore stack was unbalanced"));
        }
        self.with_renderer_mut(loan, |renderer| {
            renderer.restore();
            Ok(())
        })?;
        self.save_count.set(self.save_count.get().saturating_sub(1));
        Ok(())
    }
}

/// Puts back the target and in-use mark a reborrowed scope replaced, on
/// return and on unwinding.
struct RestoreTarget<'a> {
    renderer: &'a ScriptedRenderer,
    previous: Option<RendererTarget>,
    in_use: bool,
}

impl Drop for RestoreTarget<'_> {
    fn drop(&mut self) {
        if let Ok(mut target) = self.renderer.target.try_borrow_mut() {
            *target = self.previous.take();
        }
        self.renderer.in_use.set(self.in_use);
    }
}

impl UserData for ScriptedRenderer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_lent(
            "drawImageMeshInstanced",
            |_,
             this,
             loan,
             (image, sampler, vertices, uvs, indices, instances): (
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
            )| {
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let mut instances = instances.borrow_mut::<ScriptedImageMeshInstances>()?;
                let vertex_count = vertices.borrow::<ScriptedVertexBuffer>()?.len();
                let uv_count = uvs.borrow::<ScriptedVertexBuffer>()?.len();
                let index_count = indices.borrow::<ScriptedTriangleBuffer>()?.len();
                // Preserve the existing approved Rust host bounds boundary.
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
                    this.with_renderer_mut(loan, |renderer| {
                        // Validation above must succeed before staged writes reach
                        // the renderer, once for all writes since its last draw.
                        instances.commit();
                        let committed_instances = instances.instances.clone();
                        // The renderer may re-enter scripting. Its retained data
                        // is independent of subsequent writes to the staging vec.
                        drop(instances);
                        renderer.draw_image_mesh_instanced(
                            Some(image),
                            sampler.0,
                            vertices.render_buffer(),
                            uvs.render_buffer(),
                            indices.render_buffer(),
                            vertex_count,
                            index_count,
                            Some(&committed_instances),
                        );
                        Ok(())
                    })
                })?
            },
        );
        methods.add_method_lent("save", |_, this, loan, ()| this.save(loan));
        methods.add_method_lent("modulateOpacity", |_, this, loan, opacity: f32| {
            this.with_renderer_mut(loan, |renderer| {
                renderer.modulate_opacity(opacity);
                Ok(())
            })
        });
        methods.add_method_lent("modulateColor", |_, this, loan, color: u32| {
            this.with_renderer_mut(loan, |renderer| {
                renderer.modulate_color(color, false);
                Ok(())
            })
        });
        methods.add_method_lent("setColorModulation", |_, this, loan, color: u32| {
            this.with_renderer_mut(loan, |renderer| {
                renderer.modulate_color(color, true);
                Ok(())
            })
        });
        methods.add_method_lent("restore", |_, this, loan, ()| this.restore(loan));
        methods.add_method_lent("transform", |_, this, loan, matrix: AnyUserData| {
            let matrix = matrix.borrow::<ScriptedMat2D>()?;
            this.with_renderer_mut(loan, |renderer| {
                renderer.transform(matrix.0);
                Ok(())
            })
        });
        methods.add_method_lent("clipPath", |_, this, loan, path: AnyUserData| {
            let mut path = path.borrow_mut::<ScriptedPath>()?;
            this.bindings.with_factory(|factory| {
                let render_path = path.render_path(factory);
                this.with_renderer_mut(loan, |renderer| {
                    renderer.clip_path(render_path);
                    Ok(())
                })
            })
        });
        methods.add_method_lent(
            "drawPath",
            |_, this, loan, (path, paint): (AnyUserData, AnyUserData)| {
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                let paint = paint.borrow::<ScriptedPaint>()?;
                this.bindings.with_factory(|factory| {
                    let render_path = path.render_path(factory);
                    this.with_renderer_mut(loan, |renderer| {
                        renderer.draw_path(render_path, paint.render_paint.borrow().as_ref());
                        Ok(())
                    })
                })
            },
        );
        methods.add_method_lent(
            "drawImage",
            |_,
             this,
             loan,
             (image, sampler, blend_mode, opacity): (
                AnyUserData,
                AnyUserData,
                String,
                f32,
            )| {
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let blend_mode = parse_blend_mode_name(&blend_mode)?;
                with_scripted_image(&image, |image| {
                    this.with_renderer_mut(loan, |renderer| {
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
        methods.add_method_lent(
            "drawImageMesh",
            |_,
             this,
             loan,
             (image, sampler, vertices, uvs, indices, blend_mode, opacity): (
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                String,
                f32,
            )| {
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
                    this.with_renderer_mut(loan, |renderer| {
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

    #[derive(Default)]
    struct InstanceRecorder {
        draws: Vec<Vec<nuxie_render_api::ImageMeshInstanceData>>,
        drew_while_editing: bool,
    }

    impl Renderer for InstanceRecorder {
        fn save(&mut self) {}
        fn restore(&mut self) {}
        fn transform(&mut self, _: nuxie_render_api::Mat2D) {}
        fn draw_path(
            &mut self,
            _: &dyn nuxie_render_api::RenderPath,
            _: &dyn nuxie_render_api::RenderPaint,
        ) {
        }
        fn clip_path(&mut self, _: &dyn nuxie_render_api::RenderPath) {}
        fn modulate_opacity(&mut self, _: f32) {}
        fn draw_image(
            &mut self,
            _: Option<&dyn nuxie_render_api::RenderImage>,
            _: nuxie_render_api::ImageSampler,
            _: nuxie_render_api::BlendMode,
            _: f32,
        ) {
        }
        fn draw_image_mesh(
            &mut self,
            _: Option<&dyn nuxie_render_api::RenderImage>,
            _: nuxie_render_api::ImageSampler,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: u32,
            _: u32,
            _: nuxie_render_api::BlendMode,
            _: f32,
        ) {
        }
        fn draw_image_mesh_instanced(
            &mut self,
            _: Option<&dyn nuxie_render_api::RenderImage>,
            _: nuxie_render_api::ImageSampler,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: u32,
            _: u32,
            instances: Option<&nuxie_render_api::ImageMeshInstancesHandle>,
        ) {
            let instances = instances.unwrap().borrow();
            self.drew_while_editing |= instances.is_editing();
            self.draws.push(instances.instance_data().to_vec());
        }
    }

    // Literal script from upstream scripting_mesh_test.cpp. The final return
    // exposes the handle to the Rust harness instead of a CountingFactory;
    // its ordinary storage already counts edits and tracks end_edit balance.
    const MESH_INSTANCES_SOURCE: &str = r#"local vertices = VertexBuffer()
vertices:add(Vector.xy(0, 0), Vector.xy(1, 0), Vector.xy(1, 1))
local uvs = VertexBuffer()
uvs:add(Vector.xy(0, 0), Vector.xy(1, 0), Vector.xy(1, 1))
local triangles = TriangleBuffer()
triangles:add(0, 1, 2)
local sampler = ImageSampler('clamp', 'clamp', 'bilinear')
local instances = MeshInstances()

function fill(count: number, opacity: number)
  instances:resize(count)
  for i = 0, count - 1 do
    instances:set(i, Mat2D.withTranslation(i, 0), opacity, 0.25, Vector.xy(0.5, 0), Vector.xy(0.5, 1))
  end
end

function grow(count: number)
  for i = 0, count - 1 do
    instances:resize(i + 1)
    instances:set(i, Mat2D.withTranslation(10 + i, 0))
  end
end

function setPastEnd(): string
  instances:resize(2)
  local ok, message = pcall(function()
    instances:set(2, Mat2D.identity())
  end)
  return message
end

function render(renderer: Renderer, image: Image)
  renderer:drawImageMeshInstanced(image, sampler, vertices, uvs, triangles, instances)
end
return instances
"#;

    struct MeshInstancesTest {
        vm: ScriptVm,
        factory: PersistentFactory<RecordingFactory>,
        instances: nuxie_render_api::ImageMeshInstancesHandle,
    }
    impl MeshInstancesTest {
        fn new() -> Self {
            let vm = ScriptVm::new();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            vm.install_render_factory(&mut factory).unwrap();
            vm.install_rive_globals().unwrap();
            let instances: AnyUserData = vm.eval(MESH_INSTANCES_SOURCE).unwrap();
            let instances = instances
                .borrow::<ScriptedImageMeshInstances>()
                .unwrap()
                .instances
                .clone();
            Self {
                vm,
                factory,
                instances,
            }
        }
        fn commits(&self) -> usize {
            let instances = self.instances.borrow();
            assert!(!instances.is_editing());
            instances.edit_count()
        }
        fn fill(&self, count: usize, opacity: f32) {
            self.vm
                .lua()
                .globals()
                .get::<crate::luau::Function>("fill")
                .unwrap()
                .call::<()>((count, opacity))
                .unwrap();
        }
        fn render(&mut self, recorder: &mut InstanceRecorder) {
            let lua = self.vm.lua();
            let (renderer, mut scope) = ScriptedRenderer::create_call_scoped_userdata(
                lua,
                recorder,
                self.vm.renderer_bindings.clone(),
            )
            .unwrap();
            let image = self.factory.borrow_mut().decode_image(&[1]).unwrap();
            let image = lua
                .create_userdata(ScriptedImage::from_render_image(image))
                .unwrap();
            lua.globals()
                .get::<crate::luau::Function>("render")
                .unwrap()
                .call_lent::<()>(&mut scope, (renderer.clone(), image))
                .unwrap();
            assert!(
                renderer
                    .borrow::<ScriptedRenderer>()
                    .unwrap()
                    .end(Some(&mut scope))
            );
        }
    }

    #[test]
    fn mesh_instances_set_commits_once_per_draw() {
        let mut vm = MeshInstancesTest::new();
        let mut recorder = InstanceRecorder::default();
        vm.fill(100, 0.5);
        assert_eq!(vm.commits(), 0);
        vm.render(&mut recorder);
        assert_eq!(vm.commits(), 1);
        assert!(!recorder.drew_while_editing);
        assert_eq!(recorder.draws.len(), 1);
        assert_eq!(recorder.draws[0].len(), 100);
        let seventh = recorder.draws[0][7];
        assert_eq!(seventh.transform.0[4], 7.0);
        assert_eq!(seventh.opacity, 0.5);
        assert_eq!(seventh.additiveness, 0.25);
        assert_eq!(seventh.uv_translate, [0.5, 0.0]);
        assert_eq!(seventh.uv_scale, [0.5, 1.0]);
        vm.fill(100, 0.75);
        vm.render(&mut recorder);
        assert_eq!(vm.commits(), 2);
        assert_eq!(recorder.draws.len(), 2);
        assert_eq!(recorder.draws[1][99].opacity, 0.75);
        vm.render(&mut recorder);
        assert_eq!(vm.commits(), 2);
        assert_eq!(recorder.draws.len(), 3);
        assert_eq!(recorder.draws[2][99].opacity, 0.75);
    }

    #[test]
    fn a_renderer_userdata_reaches_only_its_own_scope_while_that_scope_is_lent() {
        // Replaces the stack-order test of the thread-local registry: scopes
        // are borrows now, so they cannot end out of order. Each userdata
        // reaches its renderer only through the loan of its own scope.
        let vm = ScriptVm::new();
        let lua = vm.lua();
        let mut first = InstanceRecorder::default();
        let mut second = InstanceRecorder::default();
        let (outer, mut outer_scope) = ScriptedRenderer::create_call_scoped_userdata(
            lua,
            &mut first,
            vm.renderer_bindings.clone(),
        )
        .unwrap();
        let (inner, mut inner_scope) = ScriptedRenderer::create_call_scoped_userdata(
            lua,
            &mut second,
            vm.renderer_bindings.clone(),
        )
        .unwrap();
        let inner_renderer = inner.borrow::<ScriptedRenderer>().unwrap();
        let outer_renderer = outer.borrow::<ScriptedRenderer>().unwrap();
        assert!(
            inner_renderer
                .with_renderer_mut(Some(&mut inner_scope), |_| Ok(()))
                .is_ok()
        );
        let error = outer_renderer
            .with_renderer_mut(Some(&mut inner_scope), |_| Ok(()))
            .unwrap_err();
        assert!(error.to_string().contains("no longer valid"), "{error}");
        assert!(
            outer_renderer
                .with_renderer_mut(Some(&mut outer_scope), |_| Ok(()))
                .is_ok()
        );
        // Without a loan (a call that did not carry the scope) neither does.
        let error = inner_renderer.with_renderer_mut(None, |_| Ok(())).unwrap_err();
        assert!(error.to_string().contains("no longer valid"), "{error}");
        assert!(inner_renderer.end(Some(&mut inner_scope)));
        assert!(
            inner_renderer
                .with_renderer_mut(Some(&mut inner_scope), |_| Ok(()))
                .is_err()
        );
    }

    #[test]
    fn mesh_instances_past_the_vm_budget_are_a_script_error() {
        use super::super::lua_mesh::MAX_IMAGE_MESH_INSTANCES;
        let vm = MeshInstancesTest::new();
        let lua = vm.vm.lua();
        lua.globals().set("limit", MAX_IMAGE_MESH_INSTANCES).unwrap();
        lua.load(
            r#"
            local a = MeshInstances()
            local ok, err = pcall(function() a:resize(limit + 1) end)
            assert(not ok and string.find(tostring(err), "at most " .. limit), tostring(err))
            ok, err = pcall(function() a:resize(4294967295) end)
            assert(not ok and string.find(tostring(err), "at most " .. limit), tostring(err))
            ok, err = pcall(function() return MeshInstances(limit + 1) end)
            assert(not ok and string.find(tostring(err), "at most " .. limit), tostring(err))
            -- The budget is per VM and across objects. A shrink refunds
            -- nothing (the renderer keeps its capacity); a drop refunds all.
            a:resize(limit)
            local b = MeshInstances()
            assert(not pcall(function() b:resize(1) end))
            a:resize(0)
            assert(not pcall(function() b:resize(1) end))
            a = nil
            b = nil
        "#,
        )
        .exec()
        .unwrap();
        lua.gc_collect().unwrap();
        lua.load(
            r#"
            local c = MeshInstances(limit)
            c:resize(limit - 1)
            c:set(limit - 2, Mat2D.identity())
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn mesh_instances_resize_keeps_earlier_sets_and_commits_once_per_draw() {
        let mut vm = MeshInstancesTest::new();
        let mut recorder = InstanceRecorder::default();
        vm.vm
            .lua()
            .globals()
            .get::<crate::luau::Function>("grow")
            .unwrap()
            .call::<()>(100)
            .unwrap();
        assert_eq!(vm.commits(), 0);
        vm.render(&mut recorder);
        assert_eq!(vm.commits(), 1);
        assert!(!recorder.drew_while_editing);
        assert_eq!(recorder.draws.len(), 1);
        assert_eq!(recorder.draws[0].len(), 100);
        for i in 0..100 {
            assert_eq!(recorder.draws[0][i].transform.0[4], 10.0 + i as f32);
        }
    }

    #[test]
    fn mesh_instances_set_rejects_an_index_past_the_end() {
        let vm = MeshInstancesTest::new();
        let message = vm
            .vm
            .lua()
            .globals()
            .get::<crate::luau::Function>("setPastEnd")
            .unwrap()
            .call::<String>(())
            .unwrap();
        assert!(
            message.contains("index 2 is past the end of MeshInstances"),
            "{message}"
        );
    }

    #[test]
    fn mesh_instances_luau_defaults_resize_set_and_instanced_draw() {
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
        let instances: AnyUserData = vm
            .eval(
                r#"
            instances = MeshInstances()
            instances:resize(2)
            instances:set(0, Mat2D.withTranslation(10, 20))
            instances:set(1, Mat2D.identity(), 0.5, 0.25, Vector.xy(0.2, 0.3), Vector.xy(0.4, 0.6))
            return instances
        "#,
            )
            .unwrap();
        {
            let instances = instances.borrow::<ScriptedImageMeshInstances>().unwrap();
            let data = instances.instances.borrow();
            // set/resize affect staging only; the renderer still has the
            // constructor's empty instance array until the first draw.
            assert_eq!(data.count(), 0);
            assert_eq!(data.edit_count(), 0);
        }
        let table: Table = vm.eval(r#"
            return {draw = function(self, renderer)
                local pts, uv, idx = VertexBuffer(), VertexBuffer(), TriangleBuffer()
                pts:add(Vector.xy(0,0), Vector.xy(1,0), Vector.xy(1,1))
                uv:add(Vector.xy(0,0), Vector.xy(1,0), Vector.xy(1,1))
                idx:add(0,1,2)
                renderer:drawImageMeshInstanced(image, ImageSampler("clamp","clamp","bilinear"), pts,uv,idx,instances)
            end}
        "#).unwrap();
        let mut renderer = factory.borrow().make_renderer();
        vm.renderer_bindings
            .call_draw(&table, &mut factory, &mut renderer, None)
            .unwrap();
        {
            let instances = instances.borrow::<ScriptedImageMeshInstances>().unwrap();
            let data = instances.instances.borrow();
            assert_eq!(data.count(), 2);
            assert_eq!(data.instance_data()[0].opacity, 1.0);
            assert_eq!(data.instance_data()[0].uv_scale, [1.0, 1.0]);
            assert_eq!(data.instance_data()[1].uv_translate, [0.2, 0.3]);
            assert_eq!(data.instance_data()[1].additiveness, 0.25);
            assert_eq!(data.edit_count(), 1);
        }
        let stream = factory.borrow().stream();
        assert_eq!(stream.matches("drawImageMeshInstanced ").count(), 1);
        assert!(!stream.contains("drawImageMesh "));
        assert!(stream.contains(
            "transform=[1,0,0,1,10,20],uvTranslate=[0,0],uvScale=[1,1],opacity=1,additiveness=0"
        ));
    }

    #[test]
    fn nested_renderer_scope_reborrows_without_unlocking_its_parent() {
        let factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let vm = ScriptVm::new();
        let lua = vm.lua();
        let (outer, mut outer_scope) = ScriptedRenderer::create_call_scoped_userdata(
            lua,
            &mut renderer,
            vm.renderer_bindings.clone(),
        )
        .unwrap();
        let outer = outer.borrow::<ScriptedRenderer>().unwrap();
        let mut nested_token = 0;
        outer
            .with_renderer_mut(Some(&mut outer_scope), |renderer| {
                renderer.save();
                {
                    let mut nested = RendererScope::new(renderer);
                    nested_token = nested.token;
                    let renderer = nested.renderer(nested_token).unwrap();
                    renderer.save();
                    // The parent's userdata is held by this host call.
                    let error = outer.with_renderer_mut(None, |_| Ok(())).unwrap_err();
                    assert!(error.to_string().contains("already mutably borrowed"), "{error}");
                    renderer.restore();
                }
                renderer.restore();
                Ok(())
            })
            .unwrap();
        assert!(outer_scope.renderer(nested_token).is_none());
        assert!(outer.end(Some(&mut outer_scope)));
        assert!(
            outer
                .with_renderer_mut(Some(&mut outer_scope), |_| Ok(()))
                .is_err()
        );
        drop(outer);
        drop(outer_scope);
        assert_eq!(
            factory.stream(),
            "rive-golden-stream-v1\nsave\nsave\nrestore\nrestore\n"
        );
    }

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
    fn a_draw_reaches_its_renderer_through_the_guarded_protected_calls_and_print() {
        // `pcall`, `xpcall` and `coroutine.resume` are host guards that call
        // the originals from Rust, and `print` runs `tostring` from Rust:
        // each passes on the draw call's loan.
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let table: Table = vm
            .eval(
                r#"
                return {
                    draw = function(self, renderer)
                        assert(pcall(function() renderer:save() end))
                        assert(xpcall(function() renderer:save() end, function(e) return e end))
                        assert(pcall(function() error("boom") end) == false)
                        assert(xpcall(function() error("boom") end, function()
                            renderer:restore()
                            return "handled"
                        end) == false)
                        assert(coroutine.resume(coroutine.create(function()
                            renderer:restore()
                        end)))
                        print(setmetatable({}, {__tostring = function()
                            renderer:save()
                            renderer:restore()
                            return "drawn"
                        end}))
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
        assert!(
            factory
                .borrow()
                .stream()
                .contains("save\nsave\nrestore\nrestore\nsave\nrestore\n"),
            "{}",
            factory.borrow().stream()
        );
    }

    #[test]
    fn instance_factory_override_draws_without_rebinding_vm_allocations() {
        let vm = ScriptVm::new();
        let mut imported_factory = PersistentFactory::new(RecordingFactory::new());
        let mut instance_factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut imported_factory).unwrap();
        vm.install_rive_globals().unwrap();
        let table: Table = vm
            .eval(
                r#"
            return {draw = function(self, renderer)
                renderer:drawPath(Path.new(), Paint.new())
                retainedRenderer = renderer
            end}
        "#,
            )
            .unwrap();
        let mut instance = vm.script_instance_from_table(table);
        let mut renderer = instance_factory.borrow().make_renderer();

        instance
            .call_draw(&mut instance_factory, &mut renderer, &mut NoopScriptHost)
            .unwrap();

        let imported_stream = imported_factory.borrow().stream();
        let instance_stream = instance_factory.borrow().stream();
        assert!(imported_stream.contains("makeRenderPaint "));
        assert!(!imported_stream.contains("drawPath "));
        assert!(instance_stream.contains("drawPath "));
        assert!(!instance_stream.contains("makeRenderPaint "));
        vm.renderer_bindings
            .verify_render_context(&mut imported_factory)
            .unwrap();
        assert!(
            !vm.eval::<bool>("return pcall(function() retainedRenderer:save() end)")
                .unwrap()
        );
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
            .call_draw_with_balance(&table, &mut factory, &mut renderer, None)
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
        let mut renderer = factory.borrow().make_renderer();
        // The approved host bounds guard rejects the Lua binding call before
        // allocation. ScriptedDrawable's outer protected call contains an
        // ordinary Lua error, so observe the binding error at this boundary.
        let error = vm
            .renderer_bindings
            .call_draw(&table, &mut factory, &mut renderer, None)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("triangle index 1 exceeds vertex buffer bounds 1"),
            "{error}"
        );
        assert!(!factory.borrow().stream().contains("makeRenderBuffer"));

        // Pinned ScriptedDrawable::draw discards the protected-call error and
        // still balances the renderer; that must not bypass the host guard.
        let mut instance = vm.script_instance_from_table(table);
        instance
            .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
            .expect("ordinary Lua draw errors are contained");
        let stream = factory.borrow().stream();
        assert!(!stream.contains("makeRenderBuffer"));
        assert!(!stream.contains("drawImageMesh"));
    }
}
