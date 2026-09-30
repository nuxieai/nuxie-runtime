//! `scripting_context_test.cpp` GPU scopes at 65638e57.
#![allow(non_snake_case)]

use super::*;
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::{OreCommandBuffer, OreCommandReader, SharedOreCommandBuffer},
    ore_commands::{CommandType, ore_payload_size_of},
    ore_render_pass_recording::RenderPassRecording,
};
use nuxie_ore_metal::{
    context::{Context, ContextApi, FrameDescriptor, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;
use std::ffi::c_void;

fn idle_canvas(vm: &ScriptVm) -> AnyUserData {
    lua_canvas::ScriptedCanvas::create(vm.lua(), vm.renderer_bindings.clone(), 0, 0).unwrap()
}

#[test]
fn open_canvas_frames_are_reclaimed_per_call_not_wholesale() {
    let vm = ScriptVm::new();
    let context = &vm.renderer_bindings;
    let outer = idle_canvas(&vm);
    let inner = idle_canvas(&vm);
    context.register_open_canvas_frame(outer.clone());
    let token = context.next_open_canvas_frame_token();
    context.register_open_canvas_frame(inner.clone());
    assert_eq!(context.take_open_canvas_frames_from(token), vec![inner]);
    assert_eq!(context.open_canvas_frame_count(), 1);
    assert_eq!(context.take_open_canvas_frames_from(0), vec![outer]);
}

#[test]
fn nested_call_swapping_inherited_frame_still_reclaims_its_own() {
    let vm = ScriptVm::new();
    let context = &vm.renderer_bindings;
    let outer = idle_canvas(&vm);
    let inner = idle_canvas(&vm);
    context.register_open_canvas_frame(outer.clone());
    let token = context.next_open_canvas_frame_token();
    context.unregister_open_canvas_frame(&outer);
    context.register_open_canvas_frame(inner.clone());
    assert_eq!(context.open_canvas_frame_count(), 1);
    assert_eq!(context.take_open_canvas_frames_from(token), vec![inner]);
    assert_eq!(context.open_canvas_frame_count(), 0);
}

#[test]
fn reclaiming_nested_frames_leaves_interleaved_outer_ones() {
    let vm = ScriptVm::new();
    let context = &vm.renderer_bindings;
    let outer11 = idle_canvas(&vm);
    let outer12 = idle_canvas(&vm);
    let inner22 = idle_canvas(&vm);
    let inner23 = idle_canvas(&vm);
    context.register_open_canvas_frame(outer11.clone());
    context.register_open_canvas_frame(outer12.clone());
    let token = context.next_open_canvas_frame_token();
    context.unregister_open_canvas_frame(&outer11);
    context.register_open_canvas_frame(inner22.clone());
    context.register_open_canvas_frame(inner23.clone());
    assert_eq!(
        context.take_open_canvas_frames_from(token),
        vec![inner22, inner23]
    );
    assert_eq!(context.open_canvas_frame_count(), 1);
    assert_eq!(context.take_open_canvas_frames_from(0), vec![outer12]);
}

#[test]
fn reclaiming_empty_open_frame_list_is_a_noop() {
    let context = RendererBindings::default();
    let token = context.next_open_canvas_frame_token();
    assert!(context.take_open_canvas_frames_from(token).is_empty());
    assert!(context.take_open_canvas_frames_from(0).is_empty());
    assert_eq!(context.open_canvas_frame_count(), 0);
}

// The portable Context constructor is private. Retain a deferred context only
// to obtain its real Context base; resource operations remain source stub no-ops.
struct StubOreContext(DeferredOreContext);
impl ContextApi for StubOreContext {
    fn contextBase(&self) -> &Context {
        self.0.contextBase()
    }
    fn features(&self) -> Features {
        self.contextBase().features()
    }
    fn lastError(&self) -> String {
        self.contextBase().lastError()
    }
    fn clearLastError(&self) {
        self.contextBase().clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.contextBase().setLastError(message);
    }
    fn makeBuffer(&mut self, _: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTexture(&mut self, _: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTextureViewImpl(&mut self, _: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeSampler(&mut self, _: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeShaderModule(&mut self, _: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeBindGroupLayout(&mut self, _: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makePipeline(
        &mut self,
        _: &PipelineDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn makeBindGroup(&mut self, _: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn beginRenderPass(
        &mut self,
        _: &RenderPassDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        None
    }
    fn beginFrame(&mut self, _: &FrameDescriptor) {}
    fn endFrame(&mut self) {}
    fn waitForGPU(&mut self) {}
    unsafe fn wrapCanvasTexture(&mut self, _: *mut c_void) -> Option<AnyResourceHandle> {
        None
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        _: *mut c_void,
        _: u32,
        _: u32,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::wgsl
    }
}

struct Fixture {
    vm: ScriptVm,
    ore: nuxie_render_api::OreContextHandle,
    errors: Rc<RefCell<Vec<String>>>,
    stream: SharedOreCommandBuffer,
}
impl Fixture {
    fn new() -> Self {
        let errors = Rc::new(RefCell::new(Vec::new()));
        let output = errors.clone();
        let vm = ScriptVm::new_with_log_sink(move |_, bytes| {
            output
                .borrow_mut()
                .push(String::from_utf8_lossy(bytes).into_owned());
        });
        let ore: nuxie_render_api::OreContextHandle = Rc::new(RefCell::new(StubOreContext(
            DeferredOreContext::fromReal(None),
        )));
        let mut factory = nuxie_render_api::PersistentFactory::new(RoutedTestFactory {
            inner: nuxie_render_api::RecordingFactory::new(),
            ore: Some(ore.clone()),
            canvas_host: None,
        });
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        assert!(Rc::ptr_eq(
            &vm.renderer_bindings.ore_context().unwrap(),
            &ore
        ));
        Self {
            vm,
            ore,
            errors,
            stream: Rc::new(RefCell::new(OreCommandBuffer::default())),
        }
    }
    fn pass(&self) -> RenderPassRecording {
        RenderPassRecording::new(
            Some(self.ore.borrow().contextBase()),
            self.stream.clone(),
            &RenderPassDesc::default(),
        )
    }
    fn opcodes(&self) -> Vec<CommandType> {
        let stream = self.stream.borrow();
        let mut reader = OreCommandReader::new(stream.command_bytes(), stream.blob_bytes());
        let mut ops = Vec::new();
        while let Some(op) = reader.next::<CommandType>() {
            ops.push(op);
            reader.skip(ore_payload_size_of(op));
        }
        ops
    }
    fn enter(&self) -> crate::gpu_canvas::ScriptCallGpuScope {
        crate::gpu_canvas::enter_script_call_gpu_scope(self.vm.lua())
    }
    fn exit(&self, scope: &crate::gpu_canvas::ScriptCallGpuScope) {
        exit_script_call_gpu_scope(self.vm.lua(), scope);
    }
}

#[test]
fn nested_call_leaves_inherited_render_pass_open() {
    let fixture = Fixture::new();
    let outer = fixture.pass();
    let scope = fixture.enter();
    fixture.exit(&scope);
    assert!(!outer.isFinished());
    assert!(fixture.ore.borrow().hasOpenRenderPasses());
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn own_abandoned_render_pass_is_finished_and_reported() {
    let fixture = Fixture::new();
    let scope = fixture.enter();
    let own = fixture.pass();
    fixture.exit(&scope);
    assert!(own.isFinished());
    assert!(!fixture.ore.borrow().hasOpenRenderPasses());
    assert_eq!(
        &*fixture.errors.borrow(),
        &[
            "GPU render pass left open at script return. Call :finish() on render passes before returning."
        ]
    );
}

#[test]
fn nested_pass_reclaimed_without_finishing_outer_one() {
    let fixture = Fixture::new();
    let mut outer = fixture.pass();
    outer.draw(3, 1, 0, 0);
    let scope = fixture.enter();
    let mut inner = fixture.pass();
    inner.draw(6, 1, 0, 0);
    fixture.exit(&scope);
    assert!(inner.isFinished());
    assert!(!outer.isFinished());
    assert!(fixture.ore.borrow().hasOpenRenderPasses());
    assert_eq!(fixture.errors.borrow().len(), 1);
    outer.draw(9, 1, 0, 0);
    outer.finish();
    assert!(!fixture.ore.borrow().hasOpenRenderPasses());
    assert_eq!(
        fixture.opcodes(),
        vec![
            CommandType::beginRenderPass,
            CommandType::draw,
            CommandType::finish,
            CommandType::beginRenderPass,
            CommandType::draw,
            CommandType::draw,
            CommandType::finish
        ]
    );
}

#[test]
fn finished_render_pass_is_not_reported_left_open() {
    let fixture = Fixture::new();
    let scope = fixture.enter();
    let mut own = fixture.pass();
    own.finish();
    fixture.exit(&scope);
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn protected_call_leaves_callers_gpu_work_alone() {
    let fixture = Fixture::new();
    let outer = fixture.pass();
    fixture
        .vm
        .renderer_bindings
        .register_open_canvas_frame(idle_canvas(&fixture.vm));
    let nested = fixture.vm.lua().create_function(|_, ()| Ok(())).unwrap();
    nested.protected_call::<()>(()).unwrap();
    assert!(!outer.isFinished());
    assert!(fixture.ore.borrow().hasOpenRenderPasses());
    assert_eq!(fixture.vm.renderer_bindings.open_canvas_frame_count(), 1);
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn protected_call_reclaims_only_nested_calls_work() {
    let fixture = Fixture::new();
    let mut outer = fixture.pass();
    fixture
        .vm
        .renderer_bindings
        .register_open_canvas_frame(idle_canvas(&fixture.vm));
    let inner = Rc::new(RefCell::new(None::<RenderPassRecording>));
    let nested_pass = inner.clone();
    let ore = fixture.ore.clone();
    let stream = fixture.stream.clone();
    let bindings = fixture.vm.renderer_bindings.clone();
    let nested = fixture
        .vm
        .lua()
        .create_function(move |lua, ()| {
            *nested_pass.borrow_mut() = Some(RenderPassRecording::new(
                Some(ore.borrow().contextBase()),
                stream.clone(),
                &RenderPassDesc::default(),
            ));
            let canvas = lua_canvas::ScriptedCanvas::create(lua, bindings.clone(), 0, 0)?;
            bindings.register_open_canvas_frame(canvas);
            Ok(())
        })
        .unwrap();
    nested.protected_call::<()>(()).unwrap();
    assert!(inner.borrow().as_ref().unwrap().isFinished());
    assert!(fixture.ore.borrow().hasOpenRenderPasses());
    assert_eq!(fixture.errors.borrow().len(), 2);
    assert!(!outer.isFinished());
    assert_eq!(fixture.vm.renderer_bindings.open_canvas_frame_count(), 1);
    outer.finish();
    inner.borrow_mut().take();
    assert_eq!(
        fixture.opcodes(),
        vec![
            CommandType::beginRenderPass,
            CommandType::finish,
            CommandType::beginRenderPass,
            CommandType::finish
        ]
    );
}
