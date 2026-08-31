//! Upstream tests/unit_tests/renderer/ore_deferred_device_state_test.cpp at e949498e.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    context::{ActiveRenderPass, Context, ContextApi, FrameDescriptor, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};
use std::{
    cell::RefCell,
    ffi::c_void,
    rc::{Rc, Weak},
};
struct FakeDeviceContext {
    base: Context,
}
impl FakeDeviceContext {
    fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_context_backend_base(Features::default(), None),
        }
    }
}
impl ContextApi for FakeDeviceContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(&self) -> Option<Weak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, p: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(p);
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, m: &str) {
        self.base.setLastError(m);
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
        ShaderTarget::glsl
    }
}
#[test]
fn recording_context_reports_replay_device_capabilities() {
    let device = Rc::new(RefCell::new(FakeDeviceContext::new()));
    let mut real = device.borrow().features();
    real.colorBufferHalfFloat = true;
    real.maxSamples = 8;
    real.maxTextureSize2D = 16384;
    nuxie_ore_metal::context_backend_set_features(&device.borrow().base, real);
    {
        let recorder = DeferredOreContext::new(Some(device.clone()));
        assert!(recorder.featuresKnown());
        assert!(recorder.features().colorBufferHalfFloat);
        assert_eq!(recorder.features().maxSamples, 8);
        assert_eq!(recorder.features().maxTextureSize2D, 16384);
    }
    {
        let mut recorder = DeferredOreContext::new(None);
        recorder.bindReal(Some(device.clone()));
        assert!(recorder.featuresKnown());
        assert!(recorder.features().colorBufferHalfFloat);
        assert_eq!(recorder.features().maxSamples, 8);
    }
    {
        let recorder = DeferredOreContext::new(None);
        assert!(!recorder.featuresKnown());
        assert!(!recorder.features().colorBufferHalfFloat);
    }
}
#[test]
fn real_context_always_knows_its_capabilities() {
    let device = FakeDeviceContext::new();
    assert!(device.featuresKnown());
}
