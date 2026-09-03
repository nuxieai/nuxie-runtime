//! tests/unit_tests/renderer/deferred_canvas_import_test.cpp at 1db281b3.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::*;
use nuxie_ore_metal::{
    context::{
        ActiveRenderPass, CanvasImageInfo, CanvasTextureInfo, Context, ContextApi, FrameDescriptor,
        ShaderTarget,
    },
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};
use std::{any::Any, ffi::c_void, rc::Weak};

struct RecordingOreContext {
    base: Context,
    sample_wraps: Vec<usize>,
}

impl RecordingOreContext {
    fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_context_backend_base(Features::default(), None),
            sample_wraps: Vec::new(),
        }
    }
}

impl ContextApi for RecordingOreContext {
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
    unsafe fn wrapCanvasTexture(&mut self, _: *mut c_void) -> Option<AnyResourceHandle> {
        None
    }
    unsafe fn wrapCanvasSampleView(
        &mut self,
        canvas: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let canvas =
            nuxie_render_api::canvas_texture_owner(&canvas).expect("recorded canvas owner");
        self.sample_wraps
            .push(Rc::as_ptr(&canvas) as *const () as usize);
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

struct ImportOrderSink {
    factory: PersistentFactory<SerializingFactory>,
    ore: Rc<RefCell<RecordingOreContext>>,
    steps: Rc<RefCell<Vec<&'static str>>>,
    content_flushed: Vec<usize>,
    open_canvas: Option<usize>,
}

impl ImportOrderSink {
    fn new() -> Self {
        Self::with_steps(Rc::new(RefCell::new(Vec::new())))
    }

    fn with_steps(steps: Rc<RefCell<Vec<&'static str>>>) -> Self {
        Self {
            factory: PersistentFactory::new(SerializingFactory::new()),
            ore: Rc::new(RefCell::new(RecordingOreContext::new())),
            steps,
            content_flushed: Vec::new(),
            open_canvas: None,
        }
    }
}

impl DeferredFrameSink for ImportOrderSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        Some(self.ore.clone())
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        None
    }
    fn begin_ore_frame(&mut self) {
        self.steps.borrow_mut().push("ore");
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        _: u32,
    ) -> Option<RendererOwner> {
        self.steps.borrow_mut().push("begin");
        self.open_canvas = Some(Rc::as_ptr(&canvas) as *const () as usize);
        Some(Rc::new(RefCell::new(Box::new(
            self.factory.borrow().make_renderer(),
        ))))
    }
    fn end_canvas_content(&mut self) {
        self.steps.borrow_mut().push("content");
        self.content_flushed.push(self.open_canvas.take().unwrap());
    }
}

fn record_canvas_write_and_sample(session: &mut DeferredSession, canvas: &RenderCanvasHandle) {
    let mut renderer = nuxie_render_api::DeferredCanvasHost::begin_canvas_content(
        session,
        canvas.clone(),
        0xff000000,
    )
    .unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    renderer.draw_path(path.as_ref(), paint.as_ref());
    nuxie_render_api::DeferredCanvasHost::end_canvas_content(session, canvas);
    let image = canvas.borrow().render_image();
    let info = CanvasImageInfo {
        identity: image.image_identity(),
        width: canvas.borrow().width(),
        height: canvas.borrow().height(),
        owner: Rc::new(image) as Rc<dyn Any>,
    };
    assert!(session
        .ore_context
        .borrow_mut()
        .recordWrapCanvasImage(info)
        .is_some());
}

#[test]
fn canvas_written_and_sampled_in_one_frame_wraps_after_its_content() {
    let mut session = DeferredSession::new(None);
    let canvas = fake_canvas();
    record_canvas_write_and_sample(&mut session, &canvas);
    session.close_open_range();

    let frame = snapshot_frame(&mut session);
    let mut sink = ImportOrderSink::new();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    let identity = Rc::as_ptr(&canvas) as *const () as usize;
    assert_eq!(sink.ore.borrow().sample_wraps, [identity]);
    assert_eq!(sink.content_flushed, [identity]);
    assert_eq!(&*sink.steps.borrow(), &["begin", "content", "ore"]);
}

struct EnsureOrderCanvas {
    image: Rc<dyn RenderImage>,
    steps: Rc<RefCell<Vec<&'static str>>>,
}

impl RenderCanvas for EnsureOrderCanvas {
    fn width(&self) -> u32 {
        8
    }
    fn height(&self) -> u32 {
        8
    }
    fn render_image(&self) -> Rc<dyn RenderImage> {
        self.image.clone()
    }
    fn ensure_backing(&mut self) {
        self.steps.borrow_mut().push("ensure");
    }
    fn begin_frame(&mut self, _: u32) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        Err(RenderCanvasError::unsupported())
    }
}

#[test]
fn first_canvas_content_ensures_backing_before_beginning_the_draw() {
    let steps = Rc::new(RefCell::new(Vec::new()));
    let canvas: RenderCanvasHandle = Rc::new(RefCell::new(Box::new(EnsureOrderCanvas {
        image: Rc::new(ForeignImage::new(0, Rc::new(Cell::new(false)))),
        steps: steps.clone(),
    })));
    let mut session = DeferredSession::new(None);
    record_canvas_write_and_sample(&mut session, &canvas);
    session.close_open_range();

    let frame = snapshot_frame(&mut session);
    let mut sink = ImportOrderSink::with_steps(steps.clone());
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(&*steps.borrow(), &["ensure", "begin", "content", "ore"]);
}

#[test]
fn each_recorded_canvas_view_resolves_to_its_own_canvas() {
    let mut session = DeferredSession::new(None);
    let canvas_a = fake_canvas();
    let canvas_b = fake_canvas();
    record_canvas_write_and_sample(&mut session, &canvas_a);
    record_canvas_write_and_sample(&mut session, &canvas_b);
    session.close_open_range();

    let frame = snapshot_frame(&mut session);
    let mut sink = ImportOrderSink::new();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(
        sink.ore.borrow().sample_wraps,
        [
            Rc::as_ptr(&canvas_a) as *const () as usize,
            Rc::as_ptr(&canvas_b) as *const () as usize,
        ]
    );
}
