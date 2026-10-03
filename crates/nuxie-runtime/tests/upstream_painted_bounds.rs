//! All eight painted_bounds_test.cpp cases at 8398db31. Arena handles replace
//! source raw pointers; the same real artboard initialization/update runs.
//! The additional draw regressions follow ShapePaint/Feather at 53419065.
use nuxie_render_api::{self as render, PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        advance_flags::AdvanceFlags,
        artboard::Artboard,
        core::{CoreArena, CoreHandle, CoreObject},
        drawable::{BoundsFidelity, Drawable},
        generated::{
            component_base::ComponentBase,
            core_registry::CoreRegistry,
            shapes::{
                paint::{
                    feather_base::FeatherBase, solid_color_base::SolidColorBase,
                    stroke_base::StrokeBase, trim_path_base::TrimPathBase,
                },
                parametric_path_base::ParametricPathBase,
            },
            transform_component_base::TransformComponentBase,
        },
        math::{aabb::Aabb, mat2d::Mat2D},
        node::Node,
        scripted::scripted_drawable::ScriptedDrawable,
        shapes::{
            paint::{
                feather::Feather, fill::Fill, shape_paint::ShapePaintPathKind,
                shape_paint_path::ShapePaintPath, solid_color::SolidColor, stroke::Stroke,
                trim_path::TrimPath,
            },
            rectangle::Rectangle,
            shape::Shape,
        },
        status_code::StatusCode,
        text::text::Text,
    },
};
fn uint(h: &CoreHandle, k: u16, v: u32) {
    assert!(CoreRegistry::set_uint_handle(h, k.into(), v));
}
fn number(h: &CoreHandle, k: u16, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, k.into(), v));
}
fn approx(a: f32, b: f32) {
    assert!(
        (a - b).abs() <= 100.0 * f32::EPSILON * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
struct Scene {
    arena: CoreArena,
    artboard: CoreHandle,
    next: u32,
}
impl Scene {
    fn new() -> Self {
        let arena = CoreArena::default();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| {
                a.set_core_arena(arena.clone());
                a.add_object(Some(artboard.clone()));
            })
            .unwrap();
        Self {
            arena,
            artboard,
            next: 1,
        }
    }
    fn add(&mut self, h: &CoreHandle, parent: u32) -> u32 {
        uint(h, ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        self.artboard
            .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(h.clone())))
            .unwrap();
        let id = self.next;
        self.next += 1;
        id
    }
    fn color(&mut self, parent: u32, value: u32) {
        let c = self.arena.insert(SolidColor::default());
        assert!(CoreRegistry::set_color_handle(
            &c,
            SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
            value as i32
        ));
        self.add(&c, parent);
    }
    fn shape(&mut self, parent: u32) -> (CoreHandle, u32, u32) {
        let s = self.arena.insert(Shape::default());
        let id = self.add(&s, parent);
        let r = self.arena.insert(Rectangle::default());
        number(&r, ParametricPathBase::WIDTH_PROPERTY_KEY, 10.0);
        number(&r, ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0);
        self.add(&r, id);
        let f = self.arena.insert(Fill::default());
        let fid = self.add(&f, id);
        self.color(fid, 0xffff0000);
        (s, id, fid)
    }
    fn stroke(&mut self, parent: u32, thickness: f32) -> (CoreHandle, u32) {
        let s = self.arena.insert(Stroke::default());
        number(&s, StrokeBase::THICKNESS_PROPERTY_KEY, thickness);
        uint(&s, StrokeBase::JOIN_PROPERTY_KEY, 1);
        uint(&s, StrokeBase::CAP_PROPERTY_KEY, 0);
        let id = self.add(&s, parent);
        self.color(id, 0xff00ff00);
        (s, id)
    }
    fn initialize(&self) {
        assert_eq!(Artboard::initialize_handle(&self.artboard), StatusCode::Ok);
        Artboard::advance_handle(
            &self.artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }
}
fn bounds(h: &CoreHandle) -> (BoundsFidelity, Aabb) {
    let mut b = Aabb::default();
    let f = h.with_mut(|o| o.painted_world_bounds(&mut b)).unwrap();
    (f, b)
}
#[test]
fn unbounded_drawable_says_none() {
    let mut d = ScriptedDrawable::default();
    let mut b = Aabb::new(1.0, 2.0, 3.0, 4.0);
    let original = b;
    assert_eq!(
        CoreObject::painted_world_bounds(&mut d, &mut b),
        BoundsFidelity::None
    );
    assert_eq!(b, original);
}
#[test]
fn filled_shape_bounds_geometry() {
    let mut scene = Scene::new();
    let (shape, _, _) = scene.shape(0);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Exact);
    approx(b.left(), -5.0);
    approx(b.top(), -5.0);
    approx(b.right(), 5.0);
    approx(b.bottom(), 5.0);
}
#[test]
fn stroke_widens_by_half_thickness() {
    let mut scene = Scene::new();
    let (shape, id, _) = scene.shape(0);
    scene.stroke(id, 8.0);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Exact);
    approx(b.left(), -9.0);
    approx(b.right(), 9.0);
}
#[test]
fn miter_widens_by_limit() {
    let mut scene = Scene::new();
    let (shape, id, _) = scene.shape(0);
    let (stroke, _) = scene.stroke(id, 8.0);
    uint(&stroke, StrokeBase::JOIN_PROPERTY_KEY, 0);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Exact);
    approx(b.left(), -5.0 - 16.0);
    approx(b.right(), 5.0 + 16.0);
}
#[test]
fn feather_widens_past_shape() {
    let mut scene = Scene::new();
    let (shape, _, fill) = scene.shape(0);
    let feather = scene.arena.insert(Feather::default());
    number(&feather, FeatherBase::STRENGTH_PROPERTY_KEY, 10.0);
    scene.add(&feather, fill);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Exact);
    approx(b.left(), -20.0);
    approx(b.right(), 20.0);
}
#[test]
fn scale_scales_local_outset() {
    let mut scene = Scene::new();
    let group = scene.arena.insert(Node::default());
    let group_id = scene.add(&group, 0);
    number(&group, TransformComponentBase::SCALE_X_PROPERTY_KEY, 3.0);
    number(&group, TransformComponentBase::SCALE_Y_PROPERTY_KEY, 3.0);
    let (shape, id, _) = scene.shape(group_id);
    scene.stroke(id, 8.0);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Exact);
    approx(b.left(), -27.0);
    approx(b.right(), 27.0);
}
#[test]
fn stroke_effect_drops_to_approximate() {
    let mut scene = Scene::new();
    let (shape, id, _) = scene.shape(0);
    let (_, stroke) = scene.stroke(id, 4.0);
    let trim = scene.arena.insert(TrimPath::default());
    uint(&trim, TrimPathBase::MODE_VALUE_PROPERTY_KEY, 1);
    number(&trim, TrimPathBase::START_PROPERTY_KEY, 0.0);
    number(&trim, TrimPathBase::END_PROPERTY_KEY, 0.5);
    scene.add(&trim, stroke);
    scene.initialize();
    let (f, b) = bounds(&shape);
    assert_eq!(f, BoundsFidelity::Approximate);
    assert!(!b.is_empty_or_nan());
}
#[test]
fn text_never_exact() {
    let mut scene = Scene::new();
    let text = scene.arena.insert(Text::default());
    scene.add(&text, 0);
    scene.initialize();
    assert_ne!(bounds(&text).0, BoundsFidelity::Exact);
}

#[derive(Debug, PartialEq)]
enum PaintDrawEvent {
    Save,
    Transform([f32; 6]),
    Clip,
    Draw,
    Restore,
}

#[derive(Default)]
struct PaintCallbackRenderer {
    events: Vec<PaintDrawEvent>,
    on_save: Option<Box<dyn FnOnce()>>,
    on_transform: Option<Box<dyn FnOnce()>>,
}

impl render::Renderer for PaintCallbackRenderer {
    fn save(&mut self) {
        self.events.push(PaintDrawEvent::Save);
        if let Some(callback) = self.on_save.take() {
            callback();
        }
    }

    fn restore(&mut self) {
        self.events.push(PaintDrawEvent::Restore);
    }

    fn transform(&mut self, transform: render::Mat2D) {
        self.events.push(PaintDrawEvent::Transform(transform.0));
        if let Some(callback) = self.on_transform.take() {
            callback();
        }
    }

    fn draw_path(&mut self, _: &dyn render::RenderPath, _: &dyn render::RenderPaint) {
        self.events.push(PaintDrawEvent::Draw);
    }

    fn clip_path(&mut self, _: &dyn render::RenderPath) {
        self.events.push(PaintDrawEvent::Clip);
    }

    fn draw_image(
        &mut self,
        _: Option<&dyn render::RenderImage>,
        _: render::ImageSampler,
        _: render::BlendMode,
        _: f32,
    ) {
        panic!("the shape fixture has no image draw");
    }

    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn render::RenderImage>,
        _: render::ImageSampler,
        _: Option<&dyn render::RenderBuffer>,
        _: Option<&dyn render::RenderBuffer>,
        _: Option<&dyn render::RenderBuffer>,
        _: u32,
        _: u32,
        _: render::BlendMode,
        _: f32,
    ) {
        panic!("the shape fixture has no image mesh draw");
    }

    fn modulate_opacity(&mut self, _: f32) {
        panic!("direct Shape::draw does not modulate renderer opacity");
    }
}

#[test]
fn world_feather_reads_its_offset_after_renderer_save() {
    let mut scene = Scene::new();
    let (shape, _, fill) = scene.shape(0);
    let feather = scene.arena.insert(Feather::default());
    uint(&feather, FeatherBase::SPACE_VALUE_PROPERTY_KEY, 0);
    number(&feather, FeatherBase::OFFSET_X_PROPERTY_KEY, 5.0);
    scene.add(&feather, fill);
    scene.initialize();
    shape
        .with_downcast_mut::<Shape, _>(|shape| shape.set_needs_save_operation(true))
        .unwrap();
    let mut renderer = PaintCallbackRenderer {
        on_save: Some(Box::new(move || {
            number(&feather, FeatherBase::OFFSET_X_PROPERTY_KEY, 7.0);
        })),
        ..Default::default()
    };

    Drawable::draw_handle(&shape, &mut renderer);

    // ShapePaint::draw decides the world-offset branch before save(), but
    // reads offsetX()/offsetY() for translate() after that virtual callback.
    assert_eq!(
        renderer.events,
        [
            PaintDrawEvent::Save,
            PaintDrawEvent::Transform([1.0, 0.0, 0.0, 1.0, 7.0, 0.0]),
            PaintDrawEvent::Transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            PaintDrawEvent::Draw,
            PaintDrawEvent::Restore,
        ]
    );
    assert!(renderer.on_save.is_none());
}

#[test]
fn local_feather_predicate_and_offset_are_read_after_path_transform() {
    let mut scene = Scene::new();
    let (shape, _, fill) = scene.shape(0);
    let feather = scene.arena.insert(Feather::default());
    // No world offset is selected at entry. The local-path transform callback
    // changes both the transform space and offset before the later local arm.
    uint(&feather, FeatherBase::SPACE_VALUE_PROPERTY_KEY, 0);
    scene.add(&feather, fill);
    scene.initialize();
    shape
        .with_downcast_mut::<Shape, _>(|shape| shape.set_needs_save_operation(true))
        .unwrap();
    let mut renderer = PaintCallbackRenderer {
        on_transform: Some(Box::new(move || {
            uint(&feather, FeatherBase::SPACE_VALUE_PROPERTY_KEY, 1);
            number(&feather, FeatherBase::OFFSET_X_PROPERTY_KEY, 7.0);
        })),
        ..Default::default()
    };

    Drawable::draw_handle(&shape, &mut renderer);

    // The second feather section in pinned ShapePaint::draw reads space and
    // offsets afresh; an entry-time tuple would omit this translation.
    assert_eq!(
        renderer.events,
        [
            PaintDrawEvent::Save,
            PaintDrawEvent::Transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            PaintDrawEvent::Transform([1.0, 0.0, 0.0, 1.0, 7.0, 0.0]),
            PaintDrawEvent::Draw,
            PaintDrawEvent::Restore,
        ]
    );
    assert!(renderer.on_transform.is_none());
}

#[test]
fn inner_feather_effect_uses_parent_container_transform_not_draw_matrix() {
    let mut scene = Scene::new();
    let (shape, _, fill_id) = scene.shape(0);
    number(&shape, TransformComponentBase::SCALE_X_PROPERTY_KEY, 2.0);
    number(&shape, TransformComponentBase::SCALE_Y_PROPERTY_KEY, 2.0);
    let feather = scene.arena.insert(Feather::default());
    assert!(CoreRegistry::set_bool_handle(
        &feather,
        FeatherBase::INNER_PROPERTY_KEY.into(),
        true,
    ));
    uint(&feather, FeatherBase::SPACE_VALUE_PROPERTY_KEY, 0);
    number(&feather, FeatherBase::STRENGTH_PROPERTY_KEY, 1.0);
    number(&feather, FeatherBase::OFFSET_X_PROPERTY_KEY, 6.0);
    number(&feather, FeatherBase::OFFSET_Y_PROPERTY_KEY, 8.0);
    scene.add(&feather, fill_id);
    let trim = scene.arena.insert(TrimPath::default());
    uint(&trim, TrimPathBase::MODE_VALUE_PROPERTY_KEY, 1);
    number(&trim, TrimPathBase::START_PROPERTY_KEY, 0.0);
    number(&trim, TrimPathBase::END_PROPERTY_KEY, 0.5);
    scene.add(&trim, fill_id);
    scene.initialize();

    let fill = scene
        .artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.resolve_handle(fill_id))
        .flatten()
        .unwrap();
    let mut path = ShapePaintPath::new(true);
    shape
        .with_downcast::<Shape, _>(|shape| {
            shape.with_path_mut(ShapePaintPathKind::Local, |source| {
                path.add_shape_paint_path(source, None);
            });
        })
        .unwrap();
    assert_eq!(
        feather.with_downcast::<Feather, _>(Feather::effect_path_dirty),
        Some(true)
    );
    let draw_matrix = Mat2D::from_scale(3.0, 4.0);
    let mut renderer = PaintCallbackRenderer::default();
    fill.with_mut(|object| {
        let paint = object.as_shape_paint_behavior_mut().unwrap();
        let fill_rule = paint.fill_rule();
        paint.shape_paint_mut().draw_with_fill_rule(
            &mut renderer,
            &mut path,
            draw_matrix,
            false,
            None,
            true,
            fill_rule,
        );
    })
    .unwrap();

    assert_eq!(
        renderer.events,
        [
            PaintDrawEvent::Save,
            PaintDrawEvent::Transform(*draw_matrix.values()),
            PaintDrawEvent::Clip,
            PaintDrawEvent::Draw,
            PaintDrawEvent::Restore,
        ]
    );
    feather
        .with_downcast::<Feather, _>(|feather| {
            assert!(!feather.effect_path_dirty());
            let inner = feather.inner_path();
            let inner = inner.borrow();
            // Pinned Feather::rebuildInnerPath emits the padded rectangle
            // first (four points), then the reversed effect path translated
            // by inverse(parent world transform) * world offset. Half of the
            // 10x10 rectangle still spans [-5, 5] in each dimension. The real
            // parent's 2x scale gives offset (3, 4), not (2, 2) from draw_matrix.
            let points = inner.raw_path().points();
            assert!(points.len() > 4);
            let hole = &points[4..];
            let min_x = hole
                .iter()
                .map(|point| point.x)
                .fold(f32::INFINITY, f32::min);
            let max_x = hole
                .iter()
                .map(|point| point.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let min_y = hole
                .iter()
                .map(|point| point.y)
                .fold(f32::INFINITY, f32::min);
            let max_y = hole
                .iter()
                .map(|point| point.y)
                .fold(f32::NEG_INFINITY, f32::max);
            approx(min_x, -2.0);
            approx(max_x, 8.0);
            approx(min_y, -1.0);
            approx(max_y, 9.0);
        })
        .unwrap();
}
