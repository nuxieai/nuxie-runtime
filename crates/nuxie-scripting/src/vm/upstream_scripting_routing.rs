//! `tests/unit_tests/runtime/scripting/scripting_routing_test.cpp` at e949498e.
use super::*;
use nuxie_render_api::*;

struct StubCanvasHost;
impl DeferredCanvasHost for StubCanvasHost {
    fn begin_canvas_content(
        &mut self,
        _: RenderCanvasHandle,
        _: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        None
    }
    fn end_canvas_content(&mut self, _: &RenderCanvasHandle) {}
}

struct SessionFactory {
    inner: RecordingFactory,
    host: DeferredCanvasHostHandle,
    bound: bool,
}
impl Factory for SessionFactory {
    fn is_render_context(&self) -> bool {
        self.bound
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        Some(self.host.clone())
    }
    fn make_render_buffer(
        &mut self,
        kind: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner.make_render_buffer(kind, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(&mut self, path: RawPath, rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }
    fn decode_image(
        &mut self,
        bytes: &[u8],
    ) -> std::result::Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(bytes)
    }
}

fn import_session(
    bound: bool,
) -> (
    Rc<ScriptVm>,
    nuxie_runtime::source::file::RuntimeFileHandle,
    PersistentFactory<SessionFactory>,
    DeferredCanvasHostHandle,
) {
    let host: DeferredCanvasHostHandle = Rc::new(RefCell::new(StubCanvasHost));
    let mut factory = PersistentFactory::new(SessionFactory {
        inner: RecordingFactory::default(),
        host: host.clone(),
        bound,
    });
    let vm = Rc::new(ScriptVm::new());
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/script_advance_test.riv"),
    )
    .unwrap();
    let file = nuxie_runtime::File::import(
        &bytes,
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        Some(
            nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle::new(Box::new(
                vm.clone(),
            )),
        ),
    )
    .unwrap();
    (vm, file, factory, host)
}

#[test]
fn import_routes_canvas_host_with_device() {
    let (vm, _file, factory, host) = import_session(true);
    assert!(Rc::ptr_eq(&vm.deferred_canvas_host().unwrap(), &host));
    assert_eq!(
        vm.render_context().unwrap().identity(),
        factory.persistent_context().unwrap().identity()
    );
    assert!(!vm.render_context_is_late_bound());
}

#[test]
fn import_routes_canvas_host_before_device() {
    let (vm, _file, _factory, host) = import_session(false);
    assert!(Rc::ptr_eq(&vm.deferred_canvas_host().unwrap(), &host));
    assert!(vm.render_context_is_late_bound());
}

fn install_test_context(vm: &ScriptVm) {
    vm.install_rive_globals().unwrap();
    let (_instance, gpu) = crate::gpu_canvas::ImportedGpuCanvasInstance::new(
        vm.gpu_canvas_shaders.clone(),
        vm.renderer_bindings.clone(),
    );
    let context = view_model::ScriptedContext::new(
        Rc::new(RefCell::new(None)),
        Vec::new(),
        Rc::new(Cell::new(false)),
        Some(gpu),
    );
    vm.lua()
        .globals()
        .set("context", vm.lua().create_userdata(context).unwrap())
        .unwrap();
}

#[test]
fn sized_canvas_construction_pending_before_device() {
    let vm = ScriptVm::new();
    vm.set_deferred_canvas_host(Some(Rc::new(RefCell::new(StubCanvasHost))));
    install_test_context(&vm);
    let result: bool=vm.lua().load("local gpu=context:gpuCanvas({width=4,height=4}); local c2d=context:canvas({width=4,height=4}); return gpu~=nil and c2d~=nil").eval().unwrap();
    assert!(result);
}

#[test]
fn sized_canvas_refuses_deviceless_factory_without_host() {
    let vm = ScriptVm::new();
    install_test_context(&vm);
    assert!(
        vm.lua()
            .load("return context:gpuCanvas({width=4,height=4})")
            .eval::<Value>()
            .is_err()
    );
}
