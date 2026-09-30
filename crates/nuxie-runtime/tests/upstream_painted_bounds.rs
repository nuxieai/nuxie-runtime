//! All eight painted_bounds_test.cpp cases at 8398db31. Arena handles replace
//! source raw pointers; the same real artboard initialization/update runs.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        advance_flags::AdvanceFlags,
        artboard::Artboard,
        core::{CoreArena, CoreHandle, CoreObject},
        drawable::BoundsFidelity,
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
        math::aabb::Aabb,
        node::Node,
        scripted::scripted_drawable::ScriptedDrawable,
        shapes::{
            paint::{
                feather::Feather, fill::Fill, solid_color::SolidColor, stroke::Stroke,
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
