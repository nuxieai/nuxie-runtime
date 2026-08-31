//! The public ORE handle retains its host; the host keeps only its bare context.
use nuxie_ore_metal::{
    context::{
        ActiveRenderPass, CanvasImageInfo, CanvasTextureInfo, Context, ContextApi, FrameDescriptor,
        ShaderTarget,
    },
    gpu_resource::AnyResourceHandle,
    ore_cmd::ore_command_buffer::SharedOreCommandBuffer,
    render_pass::RenderPassApi,
    types::*,
};
use nuxie_render_api::OreContextHandle;
use std::{
    any::Any,
    cell::RefCell,
    ffi::c_void,
    rc::{Rc, Weak},
};

// Sharing the cache also preserves identity if a factory facade is cloned.
#[derive(Clone, Default)]
pub(super) struct OreContextOwnerCache(Rc<RefCell<Option<CachedContext>>>);

struct CachedContext {
    wrapper: Weak<RefCell<OwnedContext>>,
    context: Weak<RefCell<dyn ContextApi>>,
}

impl OreContextOwnerCache {
    pub(super) fn retain(&self, context: OreContextHandle, owner: Rc<dyn Any>) -> OreContextHandle {
        if let Some(cached) = self.0.borrow().as_ref() {
            if let Some(existing) = cached.wrapper.upgrade() {
                assert!(Weak::ptr_eq(&cached.context, &Rc::downgrade(&context)));
                return existing;
            }
        }
        let base = nuxie_ore_metal::share_context_backend_base(context.borrow().contextBase());
        let native = Rc::downgrade(&context);
        let wrapper = Rc::new(RefCell::new(OwnedContext {
            base,
            context,
            _owner: owner,
        }));
        *self.0.borrow_mut() = Some(CachedContext {
            wrapper: Rc::downgrade(&wrapper),
            context: native,
        });
        wrapper
    }
}

struct OwnedContext {
    // Drop projected state and the escaping native-context reference before
    // releasing the host. Its normal teardown then drops the final native
    // context reference before destroying the backing device/implementation.
    base: Context,
    context: OreContextHandle,
    _owner: Rc<dyn Any>,
}

#[allow(non_snake_case)]
impl ContextApi for OwnedContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn canvasTargetFormat(&self) -> TextureFormat {
        self.context.borrow().canvasTargetFormat()
    }
    fn isRecording(&self) -> bool {
        self.context.borrow().isRecording()
    }
    fn featuresKnown(&self) -> bool {
        self.context.borrow().featuresKnown()
    }
    fn deferredRecording(&self) -> bool {
        self.context.borrow().deferredRecording()
    }
    fn setDeferredRecording(&self, deferred: bool) {
        self.context.borrow().setDeferredRecording(deferred);
    }
    fn usesDeferredFrameReplay(&self) -> bool {
        self.context.borrow().usesDeferredFrameReplay()
    }
    fn pendingFrame(&self) -> SharedOreCommandBuffer {
        self.context.borrow().pendingFrame()
    }
    fn recordWrapCanvasImage(&mut self, image: CanvasImageInfo) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().recordWrapCanvasImage(image)
    }
    fn recordWrapImageView(
        &mut self,
        image_id: u32,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        self.context
            .borrow_mut()
            .recordWrapImageView(image_id, width, height)
    }
    unsafe fn wrapCanvasSampleView(
        &mut self,
        canvas: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.context.borrow_mut().wrapCanvasSampleView(canvas) }
    }
    unsafe fn wrapCanvasTextureInfo(
        &mut self,
        canvas: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.context.borrow_mut().wrapCanvasTextureInfo(canvas) }
    }
    unsafe fn wrapImageSampleView(
        &mut self,
        image: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.context.borrow_mut().wrapImageSampleView(image) }
    }
    fn features(&self) -> Features {
        self.context.borrow().features()
    }
    fn lastError(&self) -> String {
        self.context.borrow().lastError()
    }
    fn activeRenderPass(&self) -> Option<Weak<dyn ActiveRenderPass>> {
        self.context.borrow().activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.context.borrow().setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        let pass = self
            .context
            .borrow()
            .activeRenderPass()
            .and_then(|pass| pass.upgrade());
        if let Some(pass) = pass {
            if !pass.isFinished() {
                pass.finish();
            }
        }
    }
    fn clearLastError(&self) {
        self.context.borrow().clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.context.borrow().setLastError(message);
    }
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeBuffer(desc)
    }
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeTexture(desc)
    }
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeTextureView(desc)
    }
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeSampler(desc)
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeShaderModule(desc)
    }
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeBindGroupLayout(desc)
    }
    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        error: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makePipeline(desc, error)
    }
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        self.context.borrow_mut().makeBindGroup(desc)
    }
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        error: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        self.context.borrow_mut().beginRenderPass(desc, error)
    }
    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        self.context.borrow_mut().beginFrame(descriptor);
    }
    fn endFrame(&mut self) {
        self.context.borrow_mut().endFrame();
    }
    fn waitForGPU(&mut self) {
        self.context.borrow_mut().waitForGPU();
    }
    unsafe fn wrapCanvasTexture(&mut self, canvas: *mut c_void) -> Option<AnyResourceHandle> {
        unsafe { self.context.borrow_mut().wrapCanvasTexture(canvas) }
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe {
            self.context
                .borrow_mut()
                .wrapRiveTexture(texture, width, height)
        }
    }
    fn shaderTarget(&self) -> ShaderTarget {
        self.context.borrow().shaderTarget()
    }
}
