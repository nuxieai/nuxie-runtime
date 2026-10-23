//! tests/unit_tests/renderer/deferred_canvas_reframe_test.cpp at 12e5327b.
use super::super::{deferred_replayer::*, deferred_session::*};
use super::*;

struct CtmRenderer {
    draws: Rc<RefCell<Vec<Mat2D>>>,
    ctm: Mat2D,
    stack: Vec<Mat2D>,
}
impl Renderer for CtmRenderer {
    fn save(&mut self) {
        self.stack.push(self.ctm);
    }
    fn restore(&mut self) {
        if let Some(ctm) = self.stack.pop() {
            self.ctm = ctm;
        }
    }
    fn transform(&mut self, m: Mat2D) {
        let [a, b, c, d, e, f] = self.ctm.0;
        let [g, h, i, j, k, l] = m.0;
        self.ctm = Mat2D([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ]);
    }
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {
        self.draws.borrow_mut().push(self.ctm);
    }
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
#[derive(Default)]
struct FrameSink {
    inner: TestSink,
    frames: Vec<Rc<RefCell<Vec<Mat2D>>>>,
    opened: Vec<usize>,
    ended: usize,
}
impl DeferredFrameSink for FrameSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.inner.factory()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        None
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.inner.begin_screen_frame(target)
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        _: u32,
    ) -> Option<RendererOwner> {
        self.opened.push(Rc::as_ptr(&canvas) as usize);
        let draws = Rc::new(RefCell::new(Vec::new()));
        self.frames.push(draws.clone());
        Some(Rc::new(RefCell::new(Box::new(CtmRenderer {
            draws,
            ctm: Mat2D::IDENTITY,
            stack: Vec::new(),
        }))))
    }
    fn end_canvas_content(&mut self) {
        self.ended += 1;
    }
}
fn scale(n: f32) -> Mat2D {
    Mat2D([n, 0.0, 0.0, n, 0.0, 0.0])
}
fn script_frame(
    session: &mut DeferredSession,
    canvas: &RenderCanvasHandle,
    path: &dyn RenderPath,
    paint: &dyn RenderPaint,
) {
    let mut r = session.begin_canvas_content(canvas.clone(), 0).unwrap();
    assert_eq!(r.current_transform(), Some(Mat2D::IDENTITY));
    r.transform(scale(3.0));
    r.draw_path(path, paint);
    session.end_canvas_content(canvas);
}
#[test]
fn a_canvass_last_frame_replaces_its_earlier_ones() {
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = fake_canvas();
    let path = session.make_empty_render_path();
    let paint = session.make_render_paint();
    script_frame(&mut session, &canvas, path.as_ref(), paint.as_ref());
    script_frame(&mut session, &canvas, path.as_ref(), paint.as_ref());
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = FrameSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(*sink.frames[0].borrow(), [scale(3.0)]);
    assert_eq!(sink.ended, 1);
}
#[test]
fn a_reader_between_two_frames_of_its_source_replays_after_it() {
    let mut session = DeferredSession::with_caps(Default::default());
    let source = fake_canvas();
    let reader = fake_canvas();
    let path = session.make_empty_render_path();
    let paint = session.make_render_paint();
    for _ in 0..2 {
        let mut r = session.begin_canvas_content(reader.clone(), 0).unwrap();
        r.draw_image(
            Some(source.borrow().render_image().as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        session.end_canvas_content(&reader);
        let mut s = session.begin_canvas_content(source.clone(), 0).unwrap();
        s.draw_path(path.as_ref(), paint.as_ref());
        session.end_canvas_content(&source);
    }
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = FrameSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(
        sink.opened,
        [Rc::as_ptr(&source) as usize, Rc::as_ptr(&reader) as usize]
    );
}
#[test]
fn a_new_frame_never_extends_a_range_already_on_its_canvas() {
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = fake_canvas();
    let path = session.make_empty_render_path();
    let paint = session.make_render_paint();
    let mut r = session.begin_canvas_content(canvas.clone(), 0).unwrap();
    session.end_canvas_content(&canvas);
    r.draw_path(path.as_ref(), paint.as_ref());
    script_frame(&mut session, &canvas, path.as_ref(), paint.as_ref());
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = FrameSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(*sink.frames[0].borrow(), [scale(3.0)]);
}
#[test]
fn a_canvas_range_resumed_after_a_nested_canvas_keeps_its_frame() {
    let mut session = DeferredSession::with_caps(Default::default());
    let outer = fake_canvas();
    let inner = fake_canvas();
    let path = session.make_empty_render_path();
    let paint = session.make_render_paint();
    let mut o = session.begin_canvas_content(outer.clone(), 0).unwrap();
    o.transform(scale(2.0));
    let mut i = session.begin_canvas_content(inner.clone(), 0).unwrap();
    i.draw_path(path.as_ref(), paint.as_ref());
    session.end_canvas_content(&inner);
    o.draw_path(path.as_ref(), paint.as_ref());
    session.end_canvas_content(&outer);
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = FrameSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(sink.frames.len(), 2);
    let outer_draws: usize = sink
        .frames
        .iter()
        .map(|frame| frame.borrow().iter().filter(|&&m| m == scale(2.0)).count())
        .sum();
    assert_eq!(outer_draws, 1);
    assert_eq!(sink.ended, 2);
}
