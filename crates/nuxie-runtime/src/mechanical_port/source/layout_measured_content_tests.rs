//! UNIV-4844: a layout box whose natural size comes from drawing content
//! follows that content's live changes, whoever writes them, through one
//! check in the update pass. Layout follows one frame later, as upstream's
//! participants do.
use crate::source::{
    generated::{
        artboard_base::ArtboardBase,
        backboard_base::BackboardBase,
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        joystick_base::JoystickBase,
        layout::{
            layout_component_style_base::LayoutComponentStyleBase,
            layout_sizing_style_base::LayoutSizingStyleBase,
        },
        layout_component_base::LayoutComponentBase,
        node_base::NodeBase,
        shapes::{
            parametric_path_base::ParametricPathBase, points_path_base::PointsPathBase,
            rectangle_base::RectangleBase, shape_base::ShapeBase,
            straight_vertex_base::StraightVertexBase, vertex_base::VertexBase,
        },
        transform_component_base::TransformComponentBase,
    },
    layout::{
        layout_enums::LayoutScaleType,
        layout_style_applier::{YGFlexDirection, YGUnit},
    },
    layout_component::LayoutComponent,
};
use crate::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use std::collections::BTreeMap;

const FRAME: f32 = 1.0 / 60.0;

/// tests/support/riv_bytes.rs, for this crate's own unit tests.
#[derive(Default)]
struct Riv {
    objects: Vec<u8>,
    toc: BTreeMap<u16, u32>,
}
fn varuint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
impl Riv {
    fn object(&mut self, key: u16) -> &mut Self {
        varuint(&mut self.objects, key.into());
        self
    }
    fn end(&mut self) {
        varuint(&mut self.objects, 0);
    }
    fn uint(&mut self, key: u16, value: u64) -> &mut Self {
        self.toc.insert(key, 0);
        varuint(&mut self.objects, key.into());
        varuint(&mut self.objects, value);
        self
    }
    fn bool(&mut self, key: u16, value: bool) -> &mut Self {
        self.toc.insert(key, 0);
        varuint(&mut self.objects, key.into());
        self.objects.push(u8::from(value));
        self
    }
    fn string(&mut self, key: u16, value: &str) -> &mut Self {
        self.toc.insert(key, 1);
        varuint(&mut self.objects, key.into());
        varuint(&mut self.objects, value.len() as u64);
        self.objects.extend_from_slice(value.as_bytes());
        self
    }
    fn float(&mut self, key: u16, value: f32) -> &mut Self {
        self.toc.insert(key, 2);
        varuint(&mut self.objects, key.into());
        self.objects.extend_from_slice(&value.to_le_bytes());
        self
    }
    fn bytes(&self) -> Vec<u8> {
        let mut out = b"RIVE".to_vec();
        varuint(&mut out, File::MAJOR_VERSION as u64);
        varuint(&mut out, File::MINOR_VERSION as u64);
        varuint(&mut out, 0);
        for &key in self.toc.keys() {
            varuint(&mut out, key.into());
        }
        varuint(&mut out, 0);
        let mut packed = 0u32;
        let mut bit = 0;
        for &field in self.toc.values() {
            packed |= field << bit;
            bit += 2;
            if bit == 8 {
                out.extend_from_slice(&packed.to_le_bytes());
                packed = 0;
                bit = 0;
            }
        }
        if bit != 0 {
            out.extend_from_slice(&packed.to_le_bytes());
        }
        out.extend_from_slice(&self.objects);
        out
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Content {
    Rectangle,
    Points,
    Joystick,
}

/// What the measured box holds, and where it sits.
struct Setup {
    content: Content,
    grouped: bool,
    // None hugs; Some is a Fixed width.
    subject_width: Option<f32>,
    row_width: f32,
    sibling_width: Option<f32>,
    // min-width: 0 on the box.
    zero_minimum: bool,
}

struct Scene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    content: Content,
}

/// One artboard: a row holding the measured box (and optionally a Fixed
/// sibling), the box holding an 80-point shape, points path or joystick,
/// directly or inside a group.
fn scene(setup: Setup) -> Scene {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // Local indices: 0 artboard, 1 row, 2 subject, then the content in the
    // order written below, the sibling and the styles.
    let mut next = 3u64;
    let mut take = |count: u64| {
        let index = next;
        next += count;
        index
    };
    let group = setup.grouped.then(|| take(1));
    let content = take(1);
    let (points_path, path) = match setup.content {
        Content::Points => {
            let path = take(1);
            take(4);
            (Some(path), None)
        }
        Content::Rectangle => (None, Some(take(1))),
        Content::Joystick => (None, None),
    };
    let sibling = setup.sibling_width.map(|_| take(1));
    let row_style = take(1);
    let subject_style = take(1);

    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "A")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, setup.row_width)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 400.0)
        .end();
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, setup.row_width)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 400.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, row_style)
        .end();
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Subject")
        .float(
            LayoutComponentBase::WIDTH_PROPERTY_KEY,
            setup.subject_width.unwrap_or(0.0),
        )
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, subject_style)
        .end();
    let content_parent = group.unwrap_or(2);
    if group.is_some() {
        riv.object(NodeBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Group")
            .end();
    }
    match setup.content {
        Content::Joystick => {
            riv.object(JoystickBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, content_parent)
                .string(ComponentBase::NAME_PROPERTY_KEY, "Content")
                .float(JoystickBase::WIDTH_PROPERTY_KEY, 80.0)
                .float(JoystickBase::HEIGHT_PROPERTY_KEY, 30.0)
                .end();
        }
        Content::Rectangle | Content::Points => {
            riv.object(ShapeBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, content_parent)
                .string(ComponentBase::NAME_PROPERTY_KEY, "Content")
                .end();
        }
    }
    if let Some(points) = points_path {
        riv.object(PointsPathBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, content)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Path")
            .end();
        for (x, y) in [(0.0, 0.0), (80.0, 0.0), (80.0, 30.0), (0.0, 30.0)] {
            riv.object(StraightVertexBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, points)
                .float(VertexBase::X_PROPERTY_KEY, x)
                .float(VertexBase::Y_PROPERTY_KEY, y)
                .end();
        }
    }
    if path.is_some() {
        riv.object(RectangleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, content)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Path")
            .float(ParametricPathBase::WIDTH_PROPERTY_KEY, 80.0)
            .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, 30.0)
            .end();
    }
    if let (Some(_), Some(width)) = (sibling, setup.sibling_width) {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, width)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
            .uint(
                LayoutComponentBase::STYLE_ID_PROPERTY_KEY,
                subject_style + 1,
            )
            .end();
    }
    let style = |riv: &mut Riv, parent: u64, hug: bool, row: bool, zero_minimum: bool| {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        for key in [
            LayoutComponentStyleBase::MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::MARGIN_TOP_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::MARGIN_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::PADDING_RIGHT_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::PADDING_TOP_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::PADDING_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::GAP_HORIZONTAL_UNITS_VALUE_PROPERTY_KEY,
            LayoutComponentStyleBase::GAP_VERTICAL_UNITS_VALUE_PROPERTY_KEY,
        ] {
            riv.uint(key, YGUnit::Point as u64);
        }
        if row {
            riv.uint(
                LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                YGFlexDirection::Row as u64,
            );
        }
        if zero_minimum {
            riv.uint(
                LayoutSizingStyleBase::MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY,
                YGUnit::Point as u64,
            )
            .float(LayoutSizingStyleBase::MIN_WIDTH_PROPERTY_KEY, 0.0);
        }
        if hug {
            riv.uint(
                LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Hug as u64,
            )
            .uint(
                LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Hug as u64,
            )
            .bool(
                LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
                true,
            );
        }
        riv.end();
    };
    style(&mut riv, 1, false, true, false);
    style(
        &mut riv,
        2,
        setup.subject_width.is_none(),
        false,
        setup.zero_minimum,
    );
    if let Some(sibling) = sibling {
        style(&mut riv, sibling, false, false, false);
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the scene imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    Scene {
        _file: file,
        artboard,
        content: setup.content,
    }
}

impl Scene {
    fn named<T: crate::source::core::CoreType>(&self, name: &str) -> CoreHandle {
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<T>())
            .into_iter()
            .find(|handle| {
                handle
                    .with(|object| {
                        object
                            .as_component()
                            .is_some_and(|component| component.base.name() == name)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("{name}"))
    }
    fn content(&self) -> CoreHandle {
        match self.content {
            Content::Joystick => self.named::<crate::source::joystick::Joystick>("Content"),
            Content::Rectangle | Content::Points => {
                self.named::<crate::source::shapes::shape::Shape>("Content")
            }
        }
    }
    fn advance(&self, frames: usize) {
        for _ in 0..frames {
            self.artboard.advance_default(FRAME);
        }
    }
    /// The measured box's solved width.
    fn width(&self) -> f32 {
        self.named::<LayoutComponent>("Subject")
            .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds().width())
            .expect("Subject")
    }
}

fn grouped(content: Content) -> Scene {
    scene(Setup {
        content,
        grouped: true,
        subject_width: None,
        row_width: 1000.0,
        sibling_width: None,
        zero_minimum: false,
    })
}

/// A write through the handle setter, as animation and data binding do.
fn set(handle: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(handle, key.into(), value));
}

#[test]
fn css_scaled_group_content_resizes_the_box() {
    let f = grouped(Content::Rectangle);
    f.advance(3);
    let before = f.width();
    set(
        &f.content(),
        TransformComponentBase::SCALE_X_PROPERTY_KEY,
        2.0,
    );
    // The update pass composes the new matrix after this frame's layout, so
    // the box follows on the next frame.
    f.advance(2);
    let widths = [before, f.width(), {
        f.advance(1);
        f.width()
    }];
    assert_eq!(widths, [80.0, 160.0, 160.0]);
}

#[test]
fn css_script_scaling_group_content_resizes_the_box() {
    let f = grouped(Content::Rectangle);
    f.advance(3);
    let before = f.width();
    // A script's node setter writes the object directly, not through the
    // handle setter (src/scripting.rs).
    let mut node = crate::ScriptNode::from_component(f.content());
    node.set_scale_x(2.0);
    f.advance(2);
    assert_eq!([before, f.width()], [80.0, 160.0]);
}

#[test]
fn css_grouped_points_path_resize_resizes_the_box() {
    let f = grouped(Content::Points);
    f.advance(3);
    let before = f.width();
    let x = i32::from(VertexBase::X_PROPERTY_KEY);
    let vertices: Vec<_> = f
        .artboard
        .with_artboard(|artboard| {
            artboard.find_all_handles::<crate::source::shapes::straight_vertex::StraightVertex>()
        })
        .into_iter()
        .filter(|vertex| CoreRegistry::get_double_handle(vertex, x) == Some(80.0))
        .collect();
    assert_eq!(vertices.len(), 2);
    // Animate the right edge from 80 to 160 points, a frame at a time.
    for x in [100.0, 120.0, 140.0, 160.0] {
        for vertex in &vertices {
            set(vertex, VertexBase::X_PROPERTY_KEY, x);
        }
        f.advance(1);
    }
    f.advance(1);
    assert_eq!([before, f.width()], [80.0, 160.0]);
}

#[test]
fn css_grouped_geometry_change_resizes_the_box() {
    let f = grouped(Content::Rectangle);
    f.advance(3);
    let before = f.width();
    let path = f.named::<crate::source::shapes::rectangle::Rectangle>("Path");
    set(&path, ParametricPathBase::WIDTH_PROPERTY_KEY, 160.0);
    f.advance(2);
    assert_eq!([before, f.width()], [80.0, 160.0]);
}

#[test]
fn css_joystick_resize_refreshes_the_box_minimum() {
    // A Fixed 200-point box beside a Fixed 60-point sibling in a 100-point
    // row shrinks to its joystick's natural minimum.
    let f = scene(Setup {
        content: Content::Joystick,
        grouped: false,
        subject_width: Some(200.0),
        row_width: 100.0,
        sibling_width: Some(60.0),
        zero_minimum: false,
    });
    f.advance(3);
    let before = f.width();
    set(&f.content(), JoystickBase::WIDTH_PROPERTY_KEY, 160.0);
    f.advance(2);
    assert_eq!([before, f.width()], [80.0, 160.0]);
}

#[test]
fn css_settled_measured_content_does_no_layout_work() {
    // Grouped content, and a directly measured shape whose size layout
    // writes back on every solve.
    for scene in [
        grouped(Content::Rectangle),
        grouped(Content::Points),
        scene(Setup {
            content: Content::Rectangle,
            grouped: false,
            subject_width: None,
            row_width: 1000.0,
            sibling_width: None,
            zero_minimum: false,
        }),
    ] {
        scene.advance(5);
        let settled = Artboard::layout_pass_count();
        let mut keep_going = 0;
        for _ in 0..30 {
            keep_going += usize::from(scene.artboard.advance_default(FRAME));
        }
        assert_eq!(Artboard::layout_pass_count(), settled);
        assert_eq!(keep_going, 0);
        assert_eq!(scene.width(), 80.0);
    }
}

#[test]
fn css_animated_group_settles_one_frame_after_the_animation_stops() {
    let f = grouped(Content::Rectangle);
    f.advance(3);
    let shape = f.content();
    for frame in 1..=10 {
        set(
            &shape,
            TransformComponentBase::SCALE_X_PROPERTY_KEY,
            1.0 + frame as f32 * 0.1,
        );
        f.advance(1);
    }
    // The last frame laid out the previous scale; one more frame lays out
    // the final 2.0, and the layout then stays settled.
    let last = f.width();
    f.advance(1);
    let settled = f.width();
    let passes = Artboard::layout_pass_count();
    f.advance(10);
    assert!((last - 152.0).abs() < 0.001, "{last}");
    assert_eq!(settled, 160.0);
    assert_eq!(Artboard::layout_pass_count(), passes);
    assert_eq!(f.width(), 160.0);
}

#[test]
fn css_equal_authored_write_still_resizes_the_box() {
    // An 80-point shape with min-width: 0 shares a 100-point row with a
    // Fixed 60-point sibling; layout writes the shrunk width into the path.
    let f = scene(Setup {
        content: Content::Rectangle,
        grouped: false,
        subject_width: None,
        row_width: 100.0,
        sibling_width: Some(60.0),
        zero_minimum: true,
    });
    f.advance(3);
    let shrunk = f.width();
    assert!((shrunk - 57.142857).abs() < 0.0001, "{shrunk}");
    let path = f.named::<crate::source::shapes::rectangle::Rectangle>("Path");
    let key = i32::from(ParametricPathBase::WIDTH_PROPERTY_KEY);
    let written = CoreRegistry::get_double_handle(&path, key).expect("path width");
    assert_eq!(written, shrunk);
    // Authoring that same width makes it the natural size: 57.14 and 60
    // now share a 17.14-point deficit.
    set(&path, ParametricPathBase::WIDTH_PROPERTY_KEY, written);
    f.advance(2);
    let width = f.width();
    assert!((width - 48.780487).abs() < 0.0001, "{width}");
}
