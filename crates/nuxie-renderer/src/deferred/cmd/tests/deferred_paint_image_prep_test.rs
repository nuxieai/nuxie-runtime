//! tests/unit_tests/renderer/deferred_paint_image_prep_test.cpp at 5e3de385.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::render_context_null::{observing_factory, ObservingFactory};
use super::*;
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaintHandle;

struct ImageSpyRenderer(Rc<RefCell<Vec<bool>>>);
impl Renderer for ImageSpyRenderer {
    fn modulate_opacity(&mut self, _: f32) {}
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.0.borrow_mut().push(
            paint
                .as_any()
                .downcast_ref::<RiveRenderPaintHandle>()
                .is_some_and(|paint| !paint.source().getImageTexture().is_null()),
        );
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
}

struct LazyImageSink {
    decoded: bool,
    factory: ObservingFactory,
    screen: RendererOwner,
    draws_with_image: Rc<RefCell<Vec<bool>>>,
}
impl LazyImageSink {
    fn new() -> Self {
        let draws_with_image = Rc::new(RefCell::new(Vec::new()));
        Self {
            decoded: false,
            factory: observing_factory(64, 64).0,
            screen: Rc::new(RefCell::new(Box::new(ImageSpyRenderer(
                draws_with_image.clone(),
            )))),
            draws_with_image,
        }
    }
}
impl DeferredFrameSink for LazyImageSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.factory.persistent_context()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        Some(self.screen.clone())
    }
    fn prep_foreign_image(&mut self, image: Rc<dyn RenderImage>) -> Option<Rc<dyn RenderImage>> {
        self.decoded.then_some(image)
    }
}

struct PaintImageScene {
    session: DeferredSession,
    sink: LazyImageSink,
    replayer: DeferredReplayer,
    image: Box<dyn RenderImage>,
    paint: Box<dyn RenderPaint>,
    path: Box<dyn RenderPath>,
}
impl PaintImageScene {
    fn new() -> Self {
        let mut session = DeferredSession::with_caps(Default::default());
        let image = session
            .decode_image(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/renderer/image_paint/nomoon.png"
            )))
            .unwrap();
        let paint = session.make_render_paint();
        let mut path = session.make_empty_render_path();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 0.0);
        path.line_to(10.0, 10.0);
        path.close();
        Self {
            session,
            sink: LazyImageSink::new(),
            replayer: DeferredReplayer::default(),
            image,
            paint,
            path,
        }
    }
    fn draw(&mut self) {
        self.paint.modulated_image(
            Some(self.image.as_ref()),
            ImageSampler::default(),
            Mat2D::IDENTITY,
        );
        self.session
            .screen_renderer(0)
            .borrow_mut()
            .draw_path(self.path.as_ref(), self.paint.as_ref());
    }
    fn replay(&mut self) -> Vec<bool> {
        self.sink.draws_with_image.borrow_mut().clear();
        self.replayer
            .replay_frame(&take_frame(&mut self.session), &mut self.sink);
        self.sink.draws_with_image.borrow().clone()
    }
}

#[test]
fn a_paint_takes_its_image_on_the_first_frame_the_image_is_ready() {
    let mut scene = PaintImageScene::new();
    scene.draw();
    assert_eq!(scene.replay(), [false]);
    scene.draw();
    assert_eq!(scene.replay(), [false]);
    scene.sink.decoded = true;
    scene.draw();
    assert_eq!(scene.replay(), [true]);
}

#[test]
fn a_draw_ahead_of_a_paint_version_bump_gets_the_image_that_landed_between_frames() {
    let mut scene = PaintImageScene::new();
    scene.draw();
    assert_eq!(scene.replay(), [false]);
    scene.sink.decoded = true;
    scene.draw();
    scene.paint.color(0xff00ff00);
    scene.draw();
    assert_eq!(scene.replay(), [true, true]);
}
