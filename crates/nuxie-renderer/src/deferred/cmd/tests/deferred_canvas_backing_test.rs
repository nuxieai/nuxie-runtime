//! tests/unit_tests/renderer/deferred_canvas_backing_test.cpp at 536a4f7a.
//! Recording allocates no backing; replay uses the sink's render context.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::render_context_null::*;
use super::*;

struct BackingSink {
    base: TestSink,
    context: Option<ObservingFactory>,
    content_opened: Vec<usize>,
}

impl BackingSink {
    fn new(context: Option<ObservingFactory>) -> Self {
        Self {
            base: TestSink::default(),
            context,
            content_opened: Vec::new(),
        }
    }
}

impl DeferredFrameSink for BackingSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.base.factory()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.context
            .as_ref()
            .and_then(|context| context.persistent_context())
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.base.begin_screen_frame(target)
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        _: u32,
    ) -> Option<RendererOwner> {
        self.content_opened.push(Rc::as_ptr(&canvas) as usize);
        None
    }
}

fn record_canvas_draw(session: &mut DeferredSession, canvas: &RenderCanvasHandle) {
    let mut content = DeferredCanvasHost::begin_canvas_content(session, canvas.clone(), 0).unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    content.draw_path(path.as_ref(), paint.as_ref());
    DeferredCanvasHost::end_canvas_content(session, canvas);
    session.close_open_range();
}

fn unbacked_canvas() -> RenderCanvasHandle {
    let mut factory = observing_factory(8, 8).0;
    Rc::new(RefCell::new(
        factory.make_deferred_render_canvas(8, 8).unwrap(),
    ))
}

struct RenderContextSink(BackingSink);
impl DeferredFrameSink for RenderContextSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.0.render_context().unwrap()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.0.render_context()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        None
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear: u32,
    ) -> Option<RendererOwner> {
        self.0.begin_canvas_content(canvas, clear)
    }
}

#[test]
fn a_paint_modulated_by_a_canvas_image_gets_the_canvases_first_texture() {
    use super::super::deferred_render_resource::DeferredRenderPaint;
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_paint_hpp::RiveRenderPaintHandle;
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = unbacked_canvas();
    record_canvas_draw(&mut session, &canvas);
    let mut paint = session.make_render_paint();
    paint.modulated_image(
        Some(canvas.borrow().render_image().as_ref()),
        ImageSampler::default(),
        Mat2D::IDENTITY,
    );
    let path = session.make_empty_render_path();
    session
        .screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    assert!(!canvas.borrow().is_backed());
    let mut sink = RenderContextSink(BackingSink::new(Some(observing_factory(8, 8).0)));
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&snapshot_frame(&mut session), &mut sink);
    assert!(canvas.borrow().is_backed());
    let recorded = paint
        .as_any()
        .downcast_ref::<DeferredRenderPaint>()
        .expect("deferred paint");
    let replayed = replayer
        .table
        .paints
        .get(recorded.resource.base.id)
        .expect("replayed paint");
    let replayed = replayed.borrow();
    let replayed = replayed
        .as_any()
        .downcast_ref::<RiveRenderPaintHandle>()
        .expect("native RiveRenderPaint");
    assert!(!replayed.source().getImageTexture().is_null());
}

#[test]
fn replay_backs_a_recorded_canvas_on_the_sinks_render_context() {
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = unbacked_canvas();
    record_canvas_draw(&mut session, &canvas);
    assert!(!canvas.borrow().is_backed());

    let mut sink = BackingSink::new(Some(observing_factory(8, 8).0));
    DeferredReplayer::default().replay_frame(&snapshot_frame(&mut session), &mut sink);

    assert!(canvas.borrow().is_backed());
    assert_eq!(sink.content_opened, vec![Rc::as_ptr(&canvas) as usize]);
}

#[test]
fn a_sink_with_no_render_context_leaves_the_canvas_unbacked() {
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = unbacked_canvas();
    record_canvas_draw(&mut session, &canvas);

    let mut sink = BackingSink::new(None);
    DeferredReplayer::default().replay_frame(&snapshot_frame(&mut session), &mut sink);

    assert!(!canvas.borrow().is_backed());
    assert!(sink.content_opened.is_empty());
}
