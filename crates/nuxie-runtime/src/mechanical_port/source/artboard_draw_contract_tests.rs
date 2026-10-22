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

/// Exercise the real mask raster and coverage application for a scene.
pub(crate) fn assert_mask_draws(
    root: &CoreHandle,
    start: &RuntimeDrawableOccurrence,
    end: &RuntimeDrawableOccurrence,
) {
    let modes = cache_mask(root, start, end);
    assert_eq!(&*modes.borrow(), &[api::LayerMaskMode::InvertedAlpha]);
}

/// Populate a scene's mask cache and expose subsequent coverage applications.
pub(crate) fn cache_mask(
    root: &CoreHandle,
    start: &RuntimeDrawableOccurrence,
    end: &RuntimeDrawableOccurrence,
) -> Rc<RefCell<Vec<api::LayerMaskMode>>> {
    let mask = start.layer_mask_marker().unwrap().borrow().mask.clone();
    let modes = Rc::new(RefCell::new(Vec::new()));
    let host: api::DeferredCanvasHostHandle = Rc::new(RefCell::new(MaskHost {
        mask,
        modes: modes.clone(),
        image: Rc::new(Image(Rc::new(()))),
    }));
    let mut factory = api::PersistentFactory::new(CanvasFactory(host));
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    root.with_downcast_mut::<Artboard, _>(|artboard| artboard.set_factory(factory))
        .unwrap();
    assert!(Artboard::draw_masked_handle(
        root,
        &mut api::NullRenderer,
        start,
        end
    ));
    modes
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

#[test]
fn translated_draw_rules_skip_noncontiguous_mask_and_keep_later_mask() {
    use crate::source::{
        generated::{
            artboard_base::ArtboardBase,
            backboard_base::BackboardBase,
            component_base::ComponentBase,
            draw_rules_base::DrawRulesBase,
            draw_target_base::DrawTargetBase,
            layer_mask_base::LayerMaskBase,
            layout_component_base::LayoutComponentBase,
            node_base::NodeBase,
            shapes::{
                paint::{fill_base::FillBase, solid_color_base::SolidColorBase},
                parametric_path_base::ParametricPathBase,
                rectangle_base::RectangleBase,
                shape_base::ShapeBase,
            },
        },
        layer_mask::LayerMask,
        layout_measured_content_tests::Riv,
    };
    fn node(riv: &mut Riv, next: &mut u64, name: &str) -> u64 {
        let id = *next;
        *next += 1;
        riv.object(NodeBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .string(ComponentBase::NAME_PROPERTY_KEY, name)
            .end();
        id
    }
    fn shape(riv: &mut Riv, next: &mut u64, parent: u64, name: &str) -> u64 {
        let id = *next;
        *next += 4;
        riv.object(ShapeBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .string(ComponentBase::NAME_PROPERTY_KEY, name)
            .end();
        riv.object(RectangleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, id)
            .float(ParametricPathBase::WIDTH_PROPERTY_KEY, 10.0)
            .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0)
            .end();
        riv.object(FillBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, id)
            .end();
        riv.object(SolidColorBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, id + 2)
            .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xffff_ffff)
            .end();
        id
    }
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .end();
    let mut next = 1;
    let first_source = shape(&mut riv, &mut next, 0, "First source");
    let second_source = shape(&mut riv, &mut next, 0, "Second source");
    let second_group = node(&mut riv, &mut next, "Second group");
    let valid_member = shape(&mut riv, &mut next, second_group, "Valid member");
    let first_group = node(&mut riv, &mut next, "First group");
    let first_member = shape(&mut riv, &mut next, first_group, "First member");
    let second_member = shape(&mut riv, &mut next, first_group, "Second member");
    for (parent, source) in [(first_group, first_source), (second_group, second_source)] {
        riv.object(LayerMaskBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .uint(LayerMaskBase::SOURCE_ID_PROPERTY_KEY, source)
            .end();
        next += 1;
    }
    let unrelated = shape(&mut riv, &mut next, 0, "Unrelated");
    riv.object(DrawRulesBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, unrelated)
        .uint(DrawRulesBase::DRAW_TARGET_ID_PROPERTY_KEY, next + 1)
        .end();
    // Rive's Before placement puts this drawable after the target in the
    // bottom-first draw traversal, between the group's two members.
    riv.object(DrawTargetBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, next)
        .uint(DrawTargetBase::DRAWABLE_ID_PROPERTY_KEY, second_member)
        .uint(DrawTargetBase::PLACEMENT_VALUE_PROPERTY_KEY, 0)
        .end();
    let mut factory = api::PersistentFactory::new(api::RecordingFactory::new());
    let file = crate::File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("translated DrawRules scene imports");
    let artboard = file.with_file(crate::File::artboard_default).unwrap();
    // Sorting must neither panic in debug nor stop at the first mask.
    for _ in 0..3 {
        artboard.advance_default(0.0);
    }
    let (masks, mut current, objects) = artboard.with_artboard(|artboard| {
        (
            artboard.all_layer_masks().to_vec(),
            artboard.first_drawable(),
            artboard.objects().to_vec(),
        )
    });
    assert_eq!(masks.len(), 2);
    assert!(objects.iter().flatten().all(|object| {
        object
            .with(|object| {
                object
                    .as_layout_component()
                    .is_none_or(|layout| !layout.is_css_mode())
            })
            .unwrap()
    }));
    let handle = |id: u64| objects[id as usize].clone().unwrap();
    assert_eq!(
        masks[0]
            .with_downcast::<LayerMask, _>(LayerMask::source)
            .flatten(),
        Some(handle(first_source))
    );
    assert_eq!(
        masks[1]
            .with_downcast::<LayerMask, _>(LayerMask::source)
            .flatten(),
        Some(handle(second_source))
    );
    let mut chain = Vec::new();
    while let Some(drawable) = current {
        current = drawable.with(Drawable::prev_drawable).flatten();
        chain.push(drawable);
    }
    let at = |id| {
        chain
            .iter()
            .position(|drawable| drawable.authored_handle() == Some(handle(id)))
            .unwrap()
    };
    assert!(at(second_member) < at(unrelated) && at(unrelated) < at(first_member));
    assert!(at(first_member) < at(valid_member));
    let brackets = chain
        .iter()
        .filter(|drawable| drawable.is_mask_start())
        .collect::<Vec<_>>();
    assert_eq!(brackets.len(), 1, "only the noncontiguous mask is skipped");
    let start = brackets[0];
    assert_eq!(start.layer_mask_marker().unwrap().borrow().mask, masks[1]);
    let end = start
        .layer_mask_marker()
        .unwrap()
        .borrow()
        .paired_end
        .as_ref()
        .and_then(|end| end.upgrade())
        .unwrap();
    let start_at = chain.iter().position(|drawable| drawable == start).unwrap();
    let end_at = chain.iter().position(|drawable| drawable == &end).unwrap();
    assert!(start_at < at(valid_member) && at(valid_member) < end_at);
    assert!(
        masks[0]
            .with_downcast::<LayerMask, _>(LayerMask::source_start)
            .flatten()
            .is_none()
    );
    assert!(
        masks[1]
            .with_downcast::<LayerMask, _>(LayerMask::source_start)
            .flatten()
            .is_some()
    );
    assert_mask_draws(&artboard.core_handle(), start, &end);
}
