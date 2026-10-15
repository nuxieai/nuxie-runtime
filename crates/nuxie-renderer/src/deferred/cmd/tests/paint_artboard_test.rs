//! Canvas cases from runtime/paint_artboard_test.cpp at fa2cd6e3.
#[path = "../../../../../nuxie-runtime/tests/support/paint_artboard.rs"]
mod support;
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::render_context_null::*;
use super::*;
use nuxie_runtime::source::{
    generated::shapes::paint::paint_image_base::PaintImageBase,
    shapes::paint::paint_image::PaintImage,
};
use nuxie_runtime::{RuntimeArtboardInstanceHandle, RuntimeFactoryHandle};
use support::*;
#[derive(Clone)]
struct CountingSession {
    session: DeferredSession,
    rasters: Rc<RefCell<Vec<(u32, u32)>>>,
}
impl DeferredCanvasHost for CountingSession {
    fn make_content_canvas(&mut self, w: u32, h: u32) -> Option<RenderCanvasHandle> {
        self.session.make_content_canvas(w, h)
    }
    fn content_canvas_image(&mut self, c: &RenderCanvasHandle) -> Option<Rc<dyn RenderImage>> {
        self.session.content_canvas_image(c)
    }
    fn begin_canvas_content(
        &mut self,
        c: RenderCanvasHandle,
        color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        self.rasters
            .borrow_mut()
            .push((c.borrow().width(), c.borrow().height()));
        DeferredCanvasHost::begin_canvas_content(&mut self.session, c, color)
    }
    fn end_canvas_content(&mut self, c: &RenderCanvasHandle) {
        DeferredCanvasHost::end_canvas_content(&mut self.session, c);
    }
}
impl Factory for CountingSession {
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.session.render_context()
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        Some(Rc::new(RefCell::new(self.clone())))
    }
    fn make_render_buffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        n: usize,
    ) -> Box<dyn RenderBuffer> {
        self.session.make_render_buffer(t, f, n)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        c: &[ColorInt],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.session.make_linear_gradient(sx, sy, ex, ey, c, s)
    }
    fn make_radial_gradient(
        &mut self,
        x: f32,
        y: f32,
        r: f32,
        c: &[ColorInt],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.session.make_radial_gradient(x, y, r, c, s)
    }
    fn make_render_path(&mut self, p: RawPath, r: FillRule) -> Box<dyn RenderPath> {
        self.session.make_render_path(p, r)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.session.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.session.make_render_paint()
    }
    fn decode_image(&mut self, d: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.session.decode_image(d)
    }
}
struct Harness {
    context: ObservingFactory,
    session: CountingSession,
    factory: RuntimeFactoryHandle,
}
impl Harness {
    fn new(can_raster: bool) -> Self {
        let context = observing_factory(256, 256).0;
        let mut session = CountingSession {
            session: DeferredSession::with_caps(Default::default()),
            rasters: Default::default(),
        };
        session.session.bind_render_context(if can_raster {
            context.persistent_context()
        } else {
            None
        });
        let factory =
            RuntimeFactoryHandle::from_factory(&mut PersistentFactory::new(session.clone()))
                .unwrap();
        Self {
            context,
            session,
            factory,
        }
    }
    fn draw(&mut self, a: &RuntimeArtboardInstanceHandle, scale: f32) {
        let screen = self.session.session.screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            screen.transform(Mat2D([scale, 0.0, 0.0, scale, 0.0, 0.0]));
            a.draw_internal(&mut **screen);
            screen.restore();
        }
        self.session.session.reset_frame();
    }
    fn count(&self) -> usize {
        self.session.rasters.borrow().len()
    }
    fn instance(&self, w: f32, h: f32, mode: u32) -> (Scene, Scene, RuntimeArtboardInstanceHandle) {
        let source = Scene::source(self.factory.clone(), 100.0, 50.0);
        let host = Scene::host(
            self.factory.clone(),
            Some(source.artboard.clone()),
            w,
            h,
            mode,
        );
        let instance = host.instance();
        (source, host, instance)
    }
}
#[test]
fn a_bound_change_rerasterizes_the_source() {
    let mut h = Harness::new(true);
    let file = read_file(h.factory.clone());
    let host = file.with_file(|f| f.artboard_named("Host")).unwrap();
    let vm = bind_instance(&file, &host);
    host.bind_view_model_instance(Some(vm.clone()));
    advance(&host, 0.0);
    h.draw(&host, 1.0);
    assert_eq!(h.count(), 1);
    advance(&host, 0.016);
    h.draw(&host, 1.0);
    assert_eq!(h.count(), 1);
    red_x(&vm, 75.0);
    advance(&host, 0.016);
    h.draw(&host, 1.0);
    assert_eq!(h.count(), 2);
}
#[test]
fn a_static_source_rasterizes_once() {
    let mut h = Harness::new(true);
    let (_s, _host, a) = h.instance(10.0, 10.0, 0);
    let p = paint_of(&a);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 1);
    assert_eq!(
        p.with_downcast::<PaintImage, _>(|p| p.source.as_ref().unwrap().raster_count())
            .unwrap(),
        1
    );
    for _ in 0..3 {
        advance(&a, 0.016);
        h.draw(&a, 1.0);
    }
    assert_eq!(h.count(), 1);
}
#[test]
fn a_changed_source_rerasterizes_on_the_next_draw() {
    let mut h = Harness::new(true);
    let (_s, _host, a) = h.instance(10.0, 10.0, 0);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 1);
    let inner = inner(&paint_of(&a));
    let objects = inner.with_artboard(|a| a.objects().to_vec());
    for object in objects.into_iter().flatten() {
        if object
            .with_downcast::<nuxie_runtime::source::shapes::shape::Shape, _>(|_| ())
            .is_some()
        {
            set_x(&object, 10.0);
        }
    }
    advance(&a, 0.016);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 2);
    advance(&a, 0.016);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 2);
}
#[test]
fn the_raster_is_sized_from_how_the_source_is_viewed() {
    // Each tuple is one independent upstream SECTION, including uint32 overflow.
    for (sw, sh, w, ht, mode, scale, tile, expected) in [
        (100.0, 50.0, 1000.0, 500.0, 0, 1.0, 1.0, (1024, 512)),
        (100.0, 50.0, 1000.0, 500.0, 1, 1.0, 1.0, (128, 64)),
        (100.0, 50.0, 1000.0, 500.0, 1, 4.0, 1.0, (448, 256)),
        (100.0, 50.0, 1000.0, 500.0, 0, 1.0, 0.25, (256, 128)),
        (100.0, 50.0, 8000.0, 4000.0, 0, 1.0, 1.0, (2048, 1024)),
        (64.0, 32.0, 64.0, 32.0, 1, 67108880.0, 1.0, (2048, 1024)),
    ] {
        let mut h = Harness::new(true);
        let source = Scene::source(h.factory.clone(), sw, sh);
        let host = Scene::host(
            h.factory.clone(),
            Some(source.artboard.clone()),
            w,
            ht,
            mode,
        );
        let a = host.instance();
        if tile != 1.0 {
            let p = paint_of(&a);
            number(&p, PaintImageBase::IMAGE_SCALE_X_PROPERTY_KEY, tile);
            number(&p, PaintImageBase::IMAGE_SCALE_Y_PROPERTY_KEY, tile);
        }
        h.draw(&a, scale);
        assert_eq!(h.count(), 1);
        assert_eq!(h.session.rasters.borrow()[0], expected);
    }
}
#[test]
fn zooming_in_rerasterizes_zooming_out_waits() {
    let mut h = Harness::new(true);
    let (_s, _host, a) = h.instance(100.0, 50.0, 0);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 1);
    assert_eq!(h.session.rasters.borrow().last().unwrap().0, 128);
    h.draw(&a, 4.0);
    assert_eq!(h.count(), 2);
    assert_eq!(h.session.rasters.borrow().last().unwrap().0, 448);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 2);
}
#[test]
fn a_host_that_cannot_raster_leaves_the_paint_without_an_image() {
    let mut h = Harness::new(false);
    let (_s, _host, a) = h.instance(10.0, 10.0, 0);
    let p = paint_of(&a);
    h.draw(&a, 1.0);
    assert_eq!(h.count(), 0);
    assert_eq!(
        p.with_downcast::<PaintImage, _>(|p| p.source.as_ref().unwrap().raster_count())
            .unwrap(),
        0
    );
}
struct NullReplaySink {
    context: ObservingFactory,
    screen: Option<RendererOwner>,
    canvas: Option<RendererOwner>,
}
impl NullReplaySink {
    fn open_frame(&mut self, w: u32, h: u32) -> RendererOwner {
        self.context.borrow().resize(w, h).unwrap();
        Rc::new(RefCell::new(Box::new(
            self.context
                .borrow()
                .begin_frame(0, crate::RenderMode::RasterOrdering)
                .unwrap(),
        )))
    }
    fn flush_screen(&mut self) {
        if self.screen.take().is_some() {
            self.context
                .borrow()
                .with_backend_mut_for_test(NullBackend::flush);
        }
    }
}
impl DeferredFrameSink for NullReplaySink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.context.persistent_context().unwrap()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.context.persistent_context()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        let r = self.open_frame(256, 256);
        self.screen = Some(r.clone());
        Some(r)
    }
    fn begin_canvas_content(&mut self, c: RenderCanvasHandle, _: u32) -> Option<RendererOwner> {
        let (w, h) = {
            let c = c.borrow();
            (c.width(), c.height())
        };
        let r = self.open_frame(w, h);
        self.canvas = Some(r.clone());
        Some(r)
    }
    fn end_canvas_content(&mut self) {
        self.context
            .borrow()
            .with_backend_mut_for_test(NullBackend::flush);
        self.canvas = None;
    }
}
#[test]
fn a_paint_samples_its_canvas_on_the_first_replayed_frame() {
    use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaintHandle;
    let mut h = Harness::new(true);
    let (_s, _host, a) = h.instance(10.0, 10.0, 0);
    {
        let screen = h.session.session.screen_renderer(0);
        a.draw_internal(&mut **screen.borrow_mut());
    }
    let frame = snapshot_frame(&mut h.session.session);
    h.session.session.reset_frame();
    assert_eq!(h.count(), 1);
    let mut sink = NullReplaySink {
        context: h.context,
        screen: None,
        canvas: None,
    };
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut sink);
    sink.flush_screen();
    let mut textured = 0;
    for p in replayer.table.paints.objects.iter().flatten() {
        let p = p.borrow();
        if let Some(p) = p.as_any().downcast_ref::<RiveRenderPaintHandle>() {
            if !p.source().getImageTexture().is_null() {
                textured += 1;
            }
        }
    }
    assert_eq!(textured, 1);
}
