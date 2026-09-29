//! `scripting_context_test.cpp` additions at 54ce53ddddb5daae38514e62a626f2bbccf3c062.
#![allow(non_snake_case)]

use super::*;
use nuxie_ore_metal::{
    context::{ActiveRenderPass, Context, ContextApi, FrameDescriptor, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;
use std::{any::Any, ffi::c_void, rc::Weak};

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

#[derive(Default)]
struct StubPassState(Cell<bool>);
impl ActiveRenderPass for StubPassState {
    fn isFinished(&self) -> bool {
        self.0.get()
    }
    fn finish(&self) {
        self.0.set(true);
    }
}

#[derive(Default)]
struct StubRenderPass(Rc<StubPassState>);
impl RenderPassApi for StubRenderPass {
    fn asAny(&self) -> &dyn Any {
        self
    }
    fn asAnyMut(&mut self) -> &mut dyn Any {
        self
    }
    fn intoAny(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn activeToken(&self) -> Weak<dyn ActiveRenderPass> {
        Rc::downgrade(&(self.0.clone() as Rc<dyn ActiveRenderPass>))
    }
    fn setPipeline(&mut self, _: Option<&AnyResourceHandle>) {}
    fn setVertexBuffer(&mut self, _: u32, _: Option<&AnyResourceHandle>, _: u32) {}
    fn setIndexBuffer(&mut self, _: Option<&AnyResourceHandle>, _: IndexFormat, _: u32) {}
    fn setBindGroup(&mut self, _: u32, _: Option<&AnyResourceHandle>, _: Option<&[u32]>, _: u32) {}
    fn setViewport(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn setScissorRect(&mut self, _: u32, _: u32, _: u32, _: u32) {}
    fn setStencilReference(&mut self, _: u32) {}
    fn setBlendColor(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn draw(&mut self, _: u32, _: u32, _: u32, _: u32) {}
    fn drawIndexed(&mut self, _: u32, _: u32, _: u32, _: i32, _: u32) {}
    fn finish(&mut self) {
        self.0.finish();
    }
    fn validate(&self) {}
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
    fn activeRenderPass(&self) -> Option<Weak<dyn ActiveRenderPass>> {
        self.contextBase().activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.contextBase().setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        self.contextBase().finishActiveRenderPass();
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
    fn makeTextureView(&mut self, _: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
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
        Self { vm, ore, errors }
    }
    fn set_pass(&self, pass: &StubRenderPass) {
        self.ore.borrow().setActiveRenderPass(Some(pass));
    }
    fn has_pass(&self, pass: &StubRenderPass) -> bool {
        self.ore
            .borrow()
            .activeRenderPass()
            .is_some_and(|active| Weak::ptr_eq(&active, &pass.activeToken()))
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
    let outer = StubRenderPass::default();
    fixture.set_pass(&outer);
    let scope = fixture.enter();
    fixture.exit(&scope);
    assert!(!outer.0.isFinished());
    assert!(fixture.has_pass(&outer));
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn own_abandoned_render_pass_is_finished_and_cleared() {
    let fixture = Fixture::new();
    let scope = fixture.enter();
    let own = StubRenderPass::default();
    fixture.set_pass(&own);
    fixture.exit(&scope);
    assert!(own.0.isFinished());
    assert!(fixture.ore.borrow().activeRenderPass().is_none());
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
    let outer = StubRenderPass::default();
    fixture.set_pass(&outer);
    let scope = fixture.enter();
    let inner = StubRenderPass::default();
    fixture.set_pass(&inner);
    fixture.exit(&scope);
    assert!(inner.0.isFinished());
    assert!(!outer.0.isFinished());
    assert!(fixture.ore.borrow().activeRenderPass().is_none());
}

#[test]
fn finished_render_pass_is_not_reported_left_open() {
    let fixture = Fixture::new();
    let scope = fixture.enter();
    let mut own = StubRenderPass::default();
    fixture.set_pass(&own);
    own.finish();
    fixture.exit(&scope);
    assert!(fixture.has_pass(&own));
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn protected_call_leaves_callers_gpu_work_alone() {
    let fixture = Fixture::new();
    let outer = StubRenderPass::default();
    fixture.set_pass(&outer);
    fixture
        .vm
        .renderer_bindings
        .register_open_canvas_frame(idle_canvas(&fixture.vm));
    let nested = fixture.vm.lua().create_function(|_, ()| Ok(())).unwrap();
    nested.protected_call::<()>(()).unwrap();
    assert!(!outer.0.isFinished());
    assert!(fixture.has_pass(&outer));
    assert_eq!(fixture.vm.renderer_bindings.open_canvas_frame_count(), 1);
    assert!(fixture.errors.borrow().is_empty());
}

#[test]
fn protected_call_reclaims_only_nested_calls_work() {
    let fixture = Fixture::new();
    let outer = StubRenderPass::default();
    fixture.set_pass(&outer);
    fixture
        .vm
        .renderer_bindings
        .register_open_canvas_frame(idle_canvas(&fixture.vm));
    let inner = Rc::new(StubRenderPass::default());
    let nested_pass = inner.clone();
    let ore = fixture.ore.clone();
    let bindings = fixture.vm.renderer_bindings.clone();
    let nested = fixture
        .vm
        .lua()
        .create_function(move |lua, ()| {
            ore.borrow().setActiveRenderPass(Some(nested_pass.as_ref()));
            let canvas = lua_canvas::ScriptedCanvas::create(lua, bindings.clone(), 0, 0)?;
            bindings.register_open_canvas_frame(canvas);
            Ok(())
        })
        .unwrap();
    nested.protected_call::<()>(()).unwrap();
    assert!(inner.0.isFinished());
    assert!(fixture.ore.borrow().activeRenderPass().is_none());
    assert_eq!(fixture.errors.borrow().len(), 2);
    assert!(!outer.0.isFinished());
    assert_eq!(fixture.vm.renderer_bindings.open_canvas_frame_count(), 1);
}
