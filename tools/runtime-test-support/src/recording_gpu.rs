//! Real deferred ORE recorder with observations at the shader/pipeline seam.
use nuxie_ore_metal::{
    context::*, gpu_resource::AnyResourceHandle, render_pass::RenderPassApi, types::*,
};
use nuxie_render_api::{
    GpuCanvasShader, OreContextHandle, RenderCanvas, RenderCanvasError, RenderCanvasFrame,
    RenderImage,
};
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

pub struct RecordingGpu {
    pub context: OreContextHandle,
    pub pipelines: Rc<RefCell<Vec<(u64, u64, bool)>>>,
    pub draws: Rc<Cell<u32>>,
    pub pipeline_entries: Rc<RefCell<Vec<(String, String)>>>,
    pub draw_vertices: Rc<RefCell<Vec<u32>>>,
    pub canvas_sizes: Rc<RefCell<Vec<(u32, u32)>>>,
    ids: Rc<RefCell<BTreeMap<usize, u64>>>,
}
impl RecordingGpu {
    pub fn new() -> Self {
        let pipelines = Rc::new(RefCell::new(Vec::new()));
        let ids = Rc::new(RefCell::new(BTreeMap::new()));
        let draws = Rc::new(Cell::new(0));
        let pipeline_entries = Rc::new(RefCell::new(Vec::new()));
        let draw_vertices = Rc::new(RefCell::new(Vec::new()));
        let canvas_sizes = Rc::new(RefCell::new(Vec::new()));
        let mut inner = DeferredOreContext::fromReal(None);
        let mut canvases: Vec<nuxie_render_api::RenderCanvasHandle> = Vec::new();
        inner.setCanvasIdProvider(Some(Box::new(move |canvas| {
            if let Some(index) = canvases
                .iter()
                .position(|existing| Rc::ptr_eq(existing, &canvas))
            {
                return index as u32;
            }
            let index = canvases.len();
            canvases.push(canvas);
            index as u32
        })));
        inner.setCanvasRegistry(Some(Rc::new(RefCell::new(
            nuxie_renderer::deferred::cmd::foreign_image_registry::ForeignImageRegistry::default(),
        ))));
        let context: OreContextHandle = Rc::new(RefCell::new(ObservedContext {
            inner,
            pipelines: pipelines.clone(),
            ids: ids.clone(),
            draws: draws.clone(),
            pipeline_entries: pipeline_entries.clone(),
            draw_vertices: draw_vertices.clone(),
            canvas_sizes: canvas_sizes.clone(),
        }));
        Self {
            context,
            pipelines,
            ids,
            draws,
            pipeline_entries,
            draw_vertices,
            canvas_sizes,
        }
    }
    pub fn shader(&self, id: u64, shader: &GpuCanvasShader) -> AnyResourceHandle {
        let bytes = if shader.binding_map_bytes.is_empty() {
            [2, 1, 14, 0, 0, 0, 0, 0].as_slice()
        } else {
            shader.binding_map_bytes.as_ref()
        };
        let desc = ShaderModuleDesc {
            code: Some(shader.source.as_bytes()),
            codeSize: shader.source.len() as u32,
            language: ShaderLanguage::wgsl,
            stage: ShaderStage::autoDetect,
            bindingMapBytes: Some(bytes),
            bindingMapSize: bytes.len() as u32,
            ..ShaderModuleDesc::default()
        };
        let module = self
            .context
            .borrow_mut()
            .makeShaderModule(&desc)
            .expect("test recorder records authored shader");
        self.ids
            .borrow_mut()
            .insert(module.allocation_identity(), id);
        module
    }
}
struct ObservedContext {
    inner: DeferredOreContext,
    pipelines: Rc<RefCell<Vec<(u64, u64, bool)>>>,
    ids: Rc<RefCell<BTreeMap<usize, u64>>>,
    draws: Rc<Cell<u32>>,
    pipeline_entries: Rc<RefCell<Vec<(String, String)>>>,
    draw_vertices: Rc<RefCell<Vec<u32>>>,
    canvas_sizes: Rc<RefCell<Vec<(u32, u32)>>>,
}
impl ContextApi for ObservedContext {
    fn contextBase(&self) -> &Context {
        self.inner.contextBase()
    }
    fn isRecording(&self) -> bool {
        self.inner.isRecording()
    }
    fn featuresKnown(&self) -> bool {
        self.inner.featuresKnown()
    }
    fn features(&self) -> Features {
        self.inner.features()
    }
    fn lastError(&self) -> String {
        self.inner.lastError()
    }
    fn activeRenderPass(&self) -> Option<std::rc::Weak<dyn ActiveRenderPass>> {
        self.inner.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.inner.setActiveRenderPass(pass)
    }
    fn finishActiveRenderPass(&self) {
        self.inner.finishActiveRenderPass()
    }
    fn clearLastError(&self) {
        self.inner.clearLastError()
    }
    fn setLastError(&self, message: &str) {
        self.inner.setLastError(message)
    }
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeBuffer(desc)
    }
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeTexture(desc)
    }
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeTextureView(desc)
    }
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeSampler(desc)
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeShaderModule(desc)
    }
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeBindGroupLayout(desc)
    }
    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        error: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        let value = self.inner.makePipeline(desc, error);
        if value.is_some() {
            self.pipeline_entries.borrow_mut().push((
                desc.vertexEntryPoint.unwrap_or_default().to_owned(),
                desc.fragmentEntryPoint.unwrap_or_default().to_owned(),
            ));
        }
        if value.is_some() {
            let ids = self.ids.borrow();
            let vertex = desc.vertexModule.unwrap().allocation_identity();
            let fragment = desc
                .fragmentModule
                .map(|module| module.allocation_identity())
                .unwrap_or(vertex);
            self.pipelines
                .borrow_mut()
                .push((ids[&vertex], ids[&fragment], vertex == fragment));
        }
        value
    }
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        self.inner.makeBindGroup(desc)
    }
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        error: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        self.inner.beginRenderPass(desc, error).map(|inner| {
            Box::new(ObservedPass {
                inner,
                draws: self.draws.clone(),
                draw_vertices: self.draw_vertices.clone(),
            }) as Box<dyn RenderPassApi>
        })
    }
    fn beginFrame(&mut self, desc: &FrameDescriptor) {
        self.inner.beginFrame(desc)
    }
    fn endFrame(&mut self) {
        self.inner.endFrame()
    }
    fn waitForGPU(&mut self) {
        self.inner.waitForGPU()
    }
    unsafe fn wrapCanvasTexture(
        &mut self,
        value: *mut std::ffi::c_void,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.inner.wrapCanvasTexture(value) }
    }
    unsafe fn wrapCanvasTextureInfo(
        &mut self,
        value: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        self.canvas_sizes
            .borrow_mut()
            .push((value.width, value.height));
        unsafe { self.inner.wrapCanvasTextureInfo(value) }
    }
    unsafe fn wrapCanvasSampleView(
        &mut self,
        value: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.inner.wrapCanvasSampleView(value) }
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        value: *mut std::ffi::c_void,
        w: u32,
        h: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.inner.wrapRiveTexture(value, w, h) }
    }
    fn recordWrapCanvasImage(&mut self, value: CanvasImageInfo) -> Option<AnyResourceHandle> {
        self.inner.recordWrapCanvasImage(value)
    }
    fn recordWrapImageView(&mut self, id: u32, w: u32, h: u32) -> Option<AnyResourceHandle> {
        self.inner.recordWrapImageView(id, w, h)
    }
    fn shaderTarget(&self) -> ShaderTarget {
        // These factories prepare WebGPU/WGSL artifacts for the observed device.
        ShaderTarget::wgsl
    }
}

struct ObservedPass {
    inner: Box<dyn RenderPassApi>,
    draws: Rc<Cell<u32>>,
    draw_vertices: Rc<RefCell<Vec<u32>>>,
}
impl RenderPassApi for ObservedPass {
    fn asAny(&self) -> &dyn std::any::Any {
        self
    }
    fn asAnyMut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn intoAny(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
    fn activeToken(&self) -> std::rc::Weak<dyn ActiveRenderPass> {
        self.inner.activeToken()
    }
    fn setPipeline(&mut self, value: Option<&AnyResourceHandle>) {
        self.inner.setPipeline(value)
    }
    fn setVertexBuffer(&mut self, slot: u32, value: Option<&AnyResourceHandle>, offset: u32) {
        self.inner.setVertexBuffer(slot, value, offset)
    }
    fn setIndexBuffer(
        &mut self,
        value: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    ) {
        self.inner.setIndexBuffer(value, format, offset)
    }
    fn setBindGroup(
        &mut self,
        index: u32,
        value: Option<&AnyResourceHandle>,
        offsets: Option<&[u32]>,
        count: u32,
    ) {
        self.inner.setBindGroup(index, value, offsets, count)
    }
    fn setViewport(&mut self, x: f32, y: f32, w: f32, h: f32, min: f32, max: f32) {
        self.inner.setViewport(x, y, w, h, min, max)
    }
    fn setScissorRect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.inner.setScissorRect(x, y, w, h)
    }
    fn setStencilReference(&mut self, value: u32) {
        self.inner.setStencilReference(value)
    }
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.setBlendColor(r, g, b, a)
    }
    fn draw(&mut self, count: u32, instances: u32, first: u32, first_instance: u32) {
        self.draws.set(self.draws.get() + 1);
        self.draw_vertices.borrow_mut().push(count);
        self.inner.draw(count, instances, first, first_instance)
    }
    fn drawIndexed(
        &mut self,
        count: u32,
        instances: u32,
        first: u32,
        base: i32,
        first_instance: u32,
    ) {
        self.draws.set(self.draws.get() + 1);
        self.inner
            .drawIndexed(count, instances, first, base, first_instance)
    }
    fn finish(&mut self) {
        self.inner.finish()
    }
    fn validate(&self) {
        self.inner.validate()
    }
}

#[derive(Clone)]
struct Image {
    identity: Rc<()>,
    width: u32,
    height: u32,
}
impl RenderImage for Image {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}
struct Canvas {
    image: Rc<Image>,
}
impl RenderCanvas for Canvas {
    fn width(&self) -> u32 {
        self.image.width
    }
    fn height(&self) -> u32 {
        self.image.height
    }
    fn render_image(&self) -> Rc<dyn RenderImage> {
        self.image.clone()
    }
    fn begin_frame(
        &mut self,
        _: nuxie_render_api::ColorInt,
    ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        Err(RenderCanvasError::new(
            "GPU recorder must not open an immediate canvas frame",
        ))
    }
}
pub fn canvas(width: u32, height: u32) -> Box<dyn RenderCanvas> {
    Box::new(Canvas {
        image: Rc::new(Image {
            identity: Rc::new(()),
            width,
            height,
        }),
    })
}
