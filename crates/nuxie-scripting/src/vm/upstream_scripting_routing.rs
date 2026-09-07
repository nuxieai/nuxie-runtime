//! `tests/unit_tests/runtime/scripting/scripting_routing_test.cpp` at dee5342a.
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
    ore: OreContextHandle,
    device: Option<PersistentFactoryContext>,
}
impl Factory for SessionFactory {
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.device.clone()
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        Some(self.host.clone())
    }
    fn ore(&mut self) -> Option<OreContextHandle> {
        Some(self.ore.clone())
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
    let ore: OreContextHandle = Rc::new(RefCell::new(
        nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::fromReal(None),
    ));
    let mut factory = PersistentFactory::new(SessionFactory {
        inner: RecordingFactory::default(),
        host: host.clone(),
        ore,
        device: bound.then(|| {
            PersistentFactory::new(RecordingFactory::new())
                .persistent_context()
                .unwrap()
        }),
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
fn caller_supplied_vm_routes_through_import_factory() {
    let vm = Rc::new(ScriptVm::new());
    let mut construction_factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut construction_factory)
        .unwrap();
    let construction_identity = construction_factory
        .persistent_context()
        .unwrap()
        .identity();

    let host: DeferredCanvasHostHandle = Rc::new(RefCell::new(StubCanvasHost));
    let ore: OreContextHandle = Rc::new(RefCell::new(
        nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::fromReal(None),
    ));
    let mut import_factory = PersistentFactory::new(SessionFactory {
        inner: RecordingFactory::new(),
        host: host.clone(),
        ore: ore.clone(),
        device: Some(
            PersistentFactory::new(RecordingFactory::new())
                .persistent_context()
                .unwrap(),
        ),
    });
    let import_identity = import_factory.persistent_context().unwrap().identity();
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/script_advance_test.riv"),
    )
    .unwrap();
    nuxie_runtime::File::import(
        &bytes,
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut import_factory).unwrap(),
        None,
        None,
        Some(
            nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle::new(Box::new(
                vm.clone(),
            )),
        ),
    )
    .expect("caller-supplied VM import");

    assert_ne!(construction_identity, import_identity);
    let device_identity = import_factory.borrow().device.as_ref().unwrap().identity();
    assert_ne!(device_identity, import_identity);
    assert_eq!(vm.render_context().unwrap().identity(), device_identity);
    assert!(Rc::ptr_eq(&vm.deferred_canvas_host().unwrap(), &host));
    assert!(Rc::ptr_eq(&vm.ore_context().unwrap(), &ore));
}

#[test]
fn import_routes_canvas_host_with_device() {
    let (vm, _file, factory, host) = import_session(true);
    assert!(Rc::ptr_eq(&vm.deferred_canvas_host().unwrap(), &host));
    assert!(vm.ore_context().is_some());
    assert_eq!(
        vm.render_context().unwrap().identity(),
        factory.borrow().device.as_ref().unwrap().identity()
    );
    assert!(!vm.render_context_is_late_bound());
}

#[test]
fn import_routes_canvas_host_before_device() {
    let (vm, _file, _factory, host) = import_session(false);
    assert!(Rc::ptr_eq(&vm.deferred_canvas_host().unwrap(), &host));
    assert!(vm.ore_context().is_some());
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
    let (vm, _file, _factory, _host) = import_session(false);
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

#[test]
fn command_server_load_routes_device_off_import_factory() {
    use nuxie_runtime::source::{
        command_queue::CommandQueue, command_server::CommandServer,
        lua::scripting_vm::RuntimeScriptingVmHandle,
    };

    // Command callbacks are Send, while the server creates and owns the VM on
    // its processing thread. Keep only test observation on that same thread;
    // no Rc or factory is transferred across the command queue boundary.
    thread_local! {
        static CREATED_VM: RefCell<Option<(Rc<ScriptVm>, RuntimeScriptingVmHandle)>> = const { RefCell::new(None) };
    }
    let host: DeferredCanvasHostHandle = Rc::new(RefCell::new(StubCanvasHost));
    let ore: OreContextHandle = Rc::new(RefCell::new(
        nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::fromReal(None),
    ));
    let device = PersistentFactory::new(RecordingFactory::new())
        .persistent_context()
        .unwrap();
    let mut factory = PersistentFactory::new(SessionFactory {
        inner: RecordingFactory::new(),
        host: host.clone(),
        ore: ore.clone(),
        device: Some(device.clone()),
    });
    let factory_identity = factory.persistent_context().unwrap().identity() as usize;
    let device_identity = device.identity() as usize;
    assert_ne!(factory_identity, device_identity);
    let host_identity = Rc::as_ptr(&host) as *const () as usize;
    let ore_identity = Rc::as_ptr(&ore) as *const () as usize;
    let mut queue = CommandQueue::default();
    let mut server = CommandServer::new(
        queue.clone(),
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
    );
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/script_advance_test.riv"),
    )
    .unwrap();
    let handle = queue.load_file(
        bytes,
        None,
        0,
        Some(Box::new(|factory| {
            let vm = Rc::new(ScriptVm::new());
            let retained = RuntimeScriptingVmHandle::new(Box::new(vm.clone()));
            retained.install_render_factory(&factory).unwrap();
            CREATED_VM.with(|slot| *slot.borrow_mut() = Some((vm, retained.clone())));
            Some(retained)
        })),
    );
    let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let callback_observed = observed.clone();
    queue.run_once(Box::new(move |server| {
        let file = server
            .get_file(handle)
            .expect("command server imported file");
        let file_vm = file
            .with_file(|file| file.scripting_vm())
            .expect("file scripting VM");
        CREATED_VM.with(|slot| {
            let slot = slot.borrow();
            let (context, retained) = slot.as_ref().expect("server created scripting context");
            assert!(file_vm.ptr_eq(retained));
            assert_eq!(
                context
                    .renderer_bindings
                    .with_factory(|factory| {
                        Ok(factory.persistent_context().unwrap().identity() as usize)
                    })
                    .unwrap(),
                factory_identity
            );
            assert_eq!(
                Rc::as_ptr(&context.deferred_canvas_host().unwrap()) as *const () as usize,
                host_identity
            );
            assert_eq!(
                Rc::as_ptr(&context.ore_context().unwrap()) as *const () as usize,
                ore_identity
            );
            assert_eq!(
                context.render_context().unwrap().identity() as usize,
                device_identity
            );
        });
        callback_observed.store(true, std::sync::atomic::Ordering::Relaxed);
    }));
    server.process_commands();
    assert!(observed.load(std::sync::atomic::Ordering::Relaxed));
    queue.disconnect();
    CREATED_VM.with(|slot| slot.borrow_mut().take());
}
