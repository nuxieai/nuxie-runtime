//! `tests/unit_tests/runtime/scripting/scripting_gpu_features_test.cpp` at e949498e.
#![allow(non_snake_case)]
use super::*;
use nuxie_ore_metal::context::{
    ActiveRenderPass, Context, ContextApi, FrameDescriptor, ShaderTarget,
};
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::types::*;
use nuxie_render_api::*;
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;

// Upstream's device stand-in advertises capabilities; factories return null.
pub(crate) struct FakeDeviceContext {
    base: Context,
    features: Features,
}
impl FakeDeviceContext {
    pub(crate) fn new(features: Features) -> Self {
        Self {
            base: nuxie_ore_metal::new_context_backend_base(features, None),
            features,
        }
    }
}
impl ContextApi for FakeDeviceContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn features(&self) -> Features {
        self.features
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(&self) -> Option<std::rc::Weak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message);
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
    unsafe fn wrapCanvasTexture(&mut self, _: *mut std::ffi::c_void) -> Option<AnyResourceHandle> {
        None
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        _: *mut std::ffi::c_void,
        _: u32,
        _: u32,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::glsl
    }
}

struct RecordingOreFactory {
    inner: RecordingFactory,
    recorder: OreContextHandle,
}

impl Factory for RecordingOreFactory {
    fn ore(&mut self) -> Option<OreContextHandle> {
        Some(self.recorder.clone())
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

fn run_with_ore_context(ore: OreContextHandle, source: &str) -> String {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingOreFactory {
        inner: RecordingFactory::new(),
        recorder: ore,
    });
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    let bindings = vm.renderer_bindings.clone();
    let features = vm
        .lua
        .create_function(move |lua, ()| bindings.gpu_features(lua))
        .unwrap();
    vm.lua.globals().set("gpuFeatures", features).unwrap();
    match vm.eval::<()>(source) {
        Ok(()) => String::new(),
        Err(error) => error.to_string(),
    }
}

#[test]
fn recording_script_reads_replay_device_capabilities() {
    let device: OreContextHandle = Rc::new(RefCell::new(FakeDeviceContext::new(Features {
        colorBufferHalfFloat: true,
        maxSamples: 8,
        ..Features::default()
    })));
    let script = "local f = gpuFeatures()\nassert(f.colorBufferHalfFloat == true, 'half float denied')\nassert(f.maxSamples == 8, 'maxSamples ' .. f.maxSamples)\n";
    let recorder = DeferredOreContext::new(Some(device.clone()));
    assert!(run_with_ore_context(Rc::new(RefCell::new(recorder)), script).is_empty());
    let mut recorder = DeferredOreContext::new(None);
    recorder.bindReal(Some(device.clone()));
    assert!(run_with_ore_context(Rc::new(RefCell::new(recorder)), script).is_empty());
    assert!(
        run_with_ore_context(
            device,
            "local f = gpuFeatures()\nassert(f.maxSamples == 8, 'maxSamples wrong')\n"
        )
        .is_empty()
    );
}

#[test]
fn unbound_recording_context_refuses_capability_readout() {
    let recorder = DeferredOreContext::new(None);
    let error = run_with_ore_context(Rc::new(RefCell::new(recorder)), "local f = gpuFeatures()");
    assert!(error.contains("context.features"));
}

#[test]
fn undecidable_capability_gate_does_not_invent_refusal() {
    let script = "local t = GPUTexture.new({ width = 4, height = 4, format = 'rgba16float', renderTarget = true })";
    let recorder = DeferredOreContext::new(None);
    assert!(
        !run_with_ore_context(Rc::new(RefCell::new(recorder)), script)
            .contains("colorBufferHalfFloat")
    );
    let capable: OreContextHandle = Rc::new(RefCell::new(FakeDeviceContext::new(Features {
        colorBufferHalfFloat: true,
        ..Features::default()
    })));
    let recorder = DeferredOreContext::new(Some(capable.clone()));
    assert!(
        !run_with_ore_context(Rc::new(RefCell::new(recorder)), script)
            .contains("colorBufferHalfFloat")
    );
    let incapable: OreContextHandle =
        Rc::new(RefCell::new(FakeDeviceContext::new(Features::default())));
    let recorder = DeferredOreContext::new(Some(incapable.clone()));
    assert!(
        run_with_ore_context(Rc::new(RefCell::new(recorder)), script)
            .contains("colorBufferHalfFloat")
    );
}
