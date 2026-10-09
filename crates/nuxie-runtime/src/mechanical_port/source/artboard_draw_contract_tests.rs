use super::*;
use api::Factory;
use nuxie_render_api as api;

struct Canvas;
impl api::RenderCanvas for Canvas {
    fn width(&self) -> u32 {
        10
    }
    fn height(&self) -> u32 {
        10
    }
    fn is_backed(&self) -> bool {
        false
    }
    fn render_image(&self) -> Rc<dyn api::RenderImage> {
        panic!("host does not request image")
    }
    fn begin_frame_with_mode(
        &mut self,
        _: u32,
        _: api::RenderCanvasFrameMode,
    ) -> Result<Box<dyn api::RenderCanvasFrame>, api::RenderCanvasError> {
        Err(api::RenderCanvasError::unsupported())
    }
}
struct Host(Rc<Cell<usize>>);
impl api::DeferredCanvasHost for Host {
    fn make_content_canvas(&mut self, _: u32, _: u32) -> Option<api::RenderCanvasHandle> {
        Some(Rc::new(RefCell::new(Box::new(Canvas))))
    }
    fn content_canvas_image(
        &mut self,
        _: &api::RenderCanvasHandle,
    ) -> Option<Rc<dyn api::RenderImage>> {
        None
    }
    fn begin_canvas_content(
        &mut self,
        _: api::RenderCanvasHandle,
        _: u32,
    ) -> Option<Box<dyn api::Renderer>> {
        self.0.set(self.0.get() + 1);
        Some(Box::new(api::NullRenderer))
    }
    fn end_canvas_content(&mut self, _: &api::RenderCanvasHandle) {}
}
struct CanvasFactory(api::DeferredCanvasHostHandle);
impl api::Factory for CanvasFactory {
    fn canvas_content_host(&mut self) -> Option<api::DeferredCanvasHostHandle> {
        Some(self.0.clone())
    }
    fn make_render_buffer(
        &mut self,
        ty: api::RenderBufferType,
        flags: api::RenderBufferFlags,
        size: usize,
    ) -> Box<dyn api::RenderBuffer> {
        api::NullFactory.make_render_buffer(ty, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[u32],
        stops: &[f32],
    ) -> Box<dyn api::RenderShader> {
        api::NullFactory.make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[u32],
        stops: &[f32],
    ) -> Box<dyn api::RenderShader> {
        api::NullFactory.make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(
        &mut self,
        raw: api::RawPath,
        fill: api::FillRule,
    ) -> Box<dyn api::RenderPath> {
        api::NullFactory.make_render_path(raw, fill)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn api::RenderPath> {
        api::NullFactory.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn api::RenderPaint> {
        api::NullFactory.make_render_paint()
    }
    fn decode_image(
        &mut self,
        bytes: &[u8],
    ) -> Result<Box<dyn api::RenderImage>, api::ImageDecodeError> {
        api::NullFactory.decode_image(bytes)
    }
}
struct QueryChangesRoot(CoreHandle);
impl api::Renderer for QueryChangesRoot {
    fn current_transform(&self) -> Option<api::Mat2D> {
        self.0
            .with_downcast_mut::<Artboard, _>(Artboard::changed)
            .unwrap();
        None
    }
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: api::Mat2D) {}
    fn draw_path(&mut self, _: &dyn api::RenderPath, _: &dyn api::RenderPaint) {}
    fn clip_path(&mut self, _: &dyn api::RenderPath) {}
    fn draw_image(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: u32,
        _: u32,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
#[test]
fn cache_reads_did_change_after_renderer_plan_queries() {
    let arena = CoreArena::default();
    let begins = Rc::new(Cell::new(0));
    let host: api::DeferredCanvasHostHandle = Rc::new(RefCell::new(Host(begins.clone())));
    let mut factory = api::PersistentFactory::new(CanvasFactory(host));
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let root = arena.insert(Artboard::with_factory(factory));
    let cache = arena.insert(BitmapCache::default());
    root.with_downcast_mut::<Artboard, _>(|a| {
        a.bitmap_cache = Some(cache);
        a.base.base.set_layout(0.0, 0.0, 10.0, 10.0);
        a.as_transform_component_mut()
            .unwrap()
            .update_render_opacity_state(None);
    })
    .unwrap();
    // Image extraction deliberately returns None; the completed content raster
    // is still cached and its begin count lets us observe the next decision.
    assert!(!Artboard::draw_cached_as_bitmap_handle(
        &root,
        &mut api::NullRenderer
    ));
    assert_eq!(begins.get(), 1);
    assert_eq!(
        root.with_downcast::<Artboard, _>(Artboard::did_change),
        Some(false)
    );
    assert!(!Artboard::draw_cached_as_bitmap_handle(
        &root,
        &mut QueryChangesRoot(root.clone())
    ));
    assert_eq!(
        begins.get(),
        2,
        "renderer query dirt must invalidate the otherwise reusable raster"
    );
    assert_eq!(
        root.with_downcast::<Artboard, _>(Artboard::did_change),
        Some(false)
    );
}

#[derive(Clone)]
struct Image(Rc<()>);
impl api::RenderImage for Image {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn api::RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
    fn width(&self) -> u32 {
        10
    }
    fn height(&self) -> u32 {
        10
    }
}
struct MaskHost {
    mask: CoreHandle,
    modes: Rc<RefCell<Vec<api::LayerMaskMode>>>,
    image: Rc<Image>,
}
impl api::DeferredCanvasHost for MaskHost {
    fn supports_layer_mask(&self) -> bool {
        true
    }
    fn make_content_canvas(&mut self, _: u32, _: u32) -> Option<api::RenderCanvasHandle> {
        Some(Rc::new(RefCell::new(Box::new(Canvas))))
    }
    fn content_canvas_image(
        &mut self,
        _: &api::RenderCanvasHandle,
    ) -> Option<Rc<dyn api::RenderImage>> {
        self.mask
            .with_downcast_mut::<crate::source::layer_mask::LayerMask, _>(|mask| {
                mask.base.set_mask_mode_value_value(1);
            })
            .unwrap();
        Some(self.image.clone())
    }
    fn begin_canvas_content(
        &mut self,
        _: api::RenderCanvasHandle,
        _: u32,
    ) -> Option<Box<dyn api::Renderer>> {
        Some(Box::new(MaskRenderer(self.modes.clone())))
    }
    fn end_canvas_content(&mut self, _: &api::RenderCanvasHandle) {}
}
struct MaskRenderer(Rc<RefCell<Vec<api::LayerMaskMode>>>);
impl api::Renderer for MaskRenderer {
    fn apply_layer_mask(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        mode: api::LayerMaskMode,
    ) {
        self.0.borrow_mut().push(mode);
    }
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: api::Mat2D) {}
    fn draw_path(&mut self, _: &dyn api::RenderPath, _: &dyn api::RenderPaint) {}
    fn clip_path(&mut self, _: &dyn api::RenderPath) {}
    fn draw_image(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: u32,
        _: u32,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
#[test]
fn mask_reads_mode_after_the_host_returns_coverage_image() {
    use crate::source::layer_mask::{LayerMask, LayerMaskOp};
    let arena = CoreArena::default();
    let mask = arena.insert(LayerMask::default());
    let modes = Rc::new(RefCell::new(Vec::new()));
    let host: api::DeferredCanvasHostHandle = Rc::new(RefCell::new(MaskHost {
        mask: mask.clone(),
        modes: modes.clone(),
        image: Rc::new(Image(Rc::new(()))),
    }));
    let mut factory = api::PersistentFactory::new(CanvasFactory(host));
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let root = arena.insert(Artboard::with_factory(factory));
    root.with_downcast_mut::<Artboard, _>(|a| {
        a.base.base.set_layout(0.0, 0.0, 10.0, 10.0);
    })
    .unwrap();
    let (start, end, source_start, source_end) = mask
        .with_downcast_mut::<LayerMask, _>(|m| {
            (
                m.create_proxy_drawable(LayerMaskOp::MaskStart),
                m.create_proxy_drawable(LayerMaskOp::MaskEnd),
                m.create_proxy_drawable(LayerMaskOp::SourceStart),
                m.create_proxy_drawable(LayerMaskOp::SourceEnd),
            )
        })
        .unwrap();
    for (first, last) in [(&start, &end), (&source_start, &source_end)] {
        let shape = arena.insert(Shape::default());
        shape
            .with_mut(|owner| {
                owner
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE);
                owner
                    .as_transform_component_mut()
                    .unwrap()
                    .update_render_opacity_state(None);
                owner.as_drawable_mut().unwrap().prev = Some(last.downgrade());
            })
            .unwrap();
        first
            .with_mut(|d| d.prev = Some(RuntimeDrawableOccurrence::Authored(shape).downgrade()))
            .unwrap();
    }
    mask.with_downcast_mut::<LayerMask, _>(|m| m.source_bracket(source_start, source_end))
        .unwrap();
    assert!(Artboard::draw_masked_handle(
        &root,
        &mut api::NullRenderer,
        &start,
        &end
    ));
    assert_eq!(&*modes.borrow(), &[api::LayerMaskMode::InvertedAlpha]);
}
