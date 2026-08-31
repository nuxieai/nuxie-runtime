//! tests/unit_tests/renderer/foreign_image_registry_test.cpp at e949498e.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::*;
type Drawn = Rc<RefCell<Vec<(usize, i32)>>>;
struct ImageRecorder(Drawn);
impl ImageRecorder {
    fn record(&self, image: Option<&dyn RenderImage>) {
        let image = image.unwrap();
        let tag = image.as_any().downcast_ref::<ForeignImage>().unwrap().0.tag;
        self.0.borrow_mut().push((image.image_identity(), tag));
    }
}
impl Renderer for ImageRecorder {
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: BlendMode,
        _: f32,
    ) {
        self.record(image);
    }
    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
        self.record(image);
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
struct ImageSink {
    base: TestSink,
    drawn: Drawn,
}
impl Default for ImageSink {
    fn default() -> Self {
        Self {
            base: TestSink::default(),
            drawn: Rc::new(RefCell::new(Vec::new())),
        }
    }
}
impl DeferredFrameSink for ImageSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.base.factory()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        Some(Rc::new(RefCell::new(Box::new(ImageRecorder(
            self.drawn.clone(),
        )))))
    }
}
fn draw_foreign(session: &DeferredSession, image: &dyn RenderImage) {
    session.screen_renderer(0).borrow_mut().draw_image(
        Some(image),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
}
fn replayed(frame: &DeferredFrame) -> Vec<(usize, i32)> {
    let mut sink = ImageSink::default();
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(frame, &mut sink);
    assert_eq!(replayer.dropped_draws(), 0);
    let drawn = sink.drawn.borrow().clone();
    drawn
}
fn replayed_inline(session: &mut DeferredSession) -> Vec<(usize, i32)> {
    let mut sink = ImageSink::default();
    let mut replayer = DeferredReplayer::default();
    replayer.replay_session(session, &mut sink);
    assert_eq!(replayer.dropped_draws(), 0);
    let drawn = sink.drawn.borrow().clone();
    drawn
}
#[test]
fn foreign_image_resolves_without_decode() {
    let destroyed = Rc::new(Cell::new(false));
    let image = ForeignImage::new(1, destroyed.clone());
    let mut first = DeferredSession::new(None);
    let mut second = DeferredSession::new(None);
    draw_foreign(&first, &image);
    let a = replayed(&take_frame(&mut first));
    draw_foreign(&second, &image);
    let b = replayed_inline(&mut second);
    let c = replayed(&take_frame(&mut second));
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(c.len(), 1);
    assert_eq!(a[0].0, image.image_identity());
    assert_eq!(b[0].0, image.image_identity());
    assert_eq!(c[0].0, image.image_identity());
    assert!(!destroyed.get());
}
#[test]
fn opposite_numbering_resolves_each_session() {
    let a = ForeignImage::new(1, Rc::new(Cell::new(false)));
    let b = ForeignImage::new(2, Rc::new(Cell::new(false)));
    let mut forward = DeferredSession::new(None);
    draw_foreign(&forward, &a);
    draw_foreign(&forward, &b);
    let fl = replayed_inline(&mut forward);
    let fs = replayed(&take_frame(&mut forward));
    let mut reverse = DeferredSession::new(None);
    draw_foreign(&reverse, &b);
    draw_foreign(&reverse, &a);
    let rl = replayed_inline(&mut reverse);
    let rs = replayed(&take_frame(&mut reverse));
    let tags = |drawn: Vec<(usize, i32)>| drawn.into_iter().map(|v| v.1).collect::<Vec<_>>();
    assert_eq!(tags(fl), [1, 2]);
    assert_eq!(tags(fs), [1, 2]);
    assert_eq!(tags(rl), [2, 1]);
    assert_eq!(tags(rs), [2, 1]);
}
#[test]
fn snapshot_retains_foreign_past_frame_and_caller() {
    let destroyed = Rc::new(Cell::new(false));
    let image = ForeignImage::new(3, destroyed.clone());
    let raw = image.image_identity();
    let mut session = DeferredSession::new(None);
    draw_foreign(&session, &image);
    let frame = take_frame(&mut session);
    assert!(Rc::strong_count(&image.0) > 1);
    drop(image);
    assert!(!destroyed.get());
    let drawn = replayed(&frame);
    assert_eq!(drawn.len(), 1);
    assert_eq!(drawn[0].0, raw);
    assert_eq!(drawn[0].1, 3);
    assert!(!destroyed.get());
    drop(frame);
    assert!(destroyed.get());
}
