//! CSS mode (docs/CSS_LAYOUT.md): a box with a `LayoutCssStyle` record is
//! laid out by its CSS, and a box without one by the CSS its Rive modes
//! translate to. The oracle is CSS arithmetic, as a browser computes it.
use super::layout_measured_content_tests::Riv;
use crate::source::{
    generated::{
        artboard_base::ArtboardBase,
        backboard_base::BackboardBase,
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        layout::{
            layout_component_style_base::LayoutComponentStyleBase,
            layout_sizing_style_base::LayoutSizingStyleBase,
        },
        layout_component_base::LayoutComponentBase,
        node_base::NodeBase,
        shapes::{
            parametric_path_base::ParametricPathBase, rectangle_base::RectangleBase,
            shape_base::ShapeBase,
        },
    },
    layout::{
        layout_enums::LayoutScaleType,
        layout_style_applier::{YGFlexDirection, YGUnit},
    },
    layout_component::LayoutComponent,
};
use crate::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

const FRAME: f32 = 1.0 / 60.0;

/// The record type and the keys these tests write (defs/nuxie/layout_css_style.json).
const LAYOUT_CSS_STYLE: u16 = 60002;
const DISPLAY: u16 = 60022;
const BOX_SIZING: u16 = 60023;
const POSITION: u16 = 60024;
const FLEX_GROW: u16 = 60030;
const FLEX_SHRINK: u16 = 60031;
const WIDTH_KIND: u16 = 60043;
const WIDTH: u16 = 60044;
const WIDTH_PERCENT: u16 = 60045;
const HEIGHT_KIND: u16 = 60046;
const HEIGHT: u16 = 60047;
const MARGIN_TOP: u16 = 60062;
const MARGIN_BOTTOM: u16 = 60068;
const INSET_TOP_KIND: u16 = 60073;
const INSET_TOP: u16 = 60074;
const INSET_LEFT_KIND: u16 = 60082;
const INSET_LEFT: u16 = 60083;
const PADDING_TOP: u16 = 60085;
const PADDING_RIGHT: u16 = 60087;
const PADDING_BOTTOM: u16 = 60089;
const PADDING_LEFT: u16 = 60091;
const BORDER_TOP: u16 = 60093;
const BORDER_RIGHT: u16 = 60094;
const BORDER_BOTTOM: u16 = 60095;
const BORDER_LEFT: u16 = 60096;
const COLUMN_GAP: u16 = 60099;

const FLEX: f32 = 1.0;
const NONE: f32 = 3.0;
const LENGTH: f32 = 1.0;

/// One property: a uint for enums and kinds, a float for numbers.
#[derive(Clone, Copy)]
enum P {
    U(u16, u64),
    F(u16, f32),
}
use P::{F, U};

fn width(value: f32) -> [P; 2] {
    [U(WIDTH_KIND, 1), F(WIDTH, value)]
}
fn height(value: f32) -> [P; 2] {
    [U(HEIGHT_KIND, 1), F(HEIGHT, value)]
}
fn padding(value: f32) -> [P; 4] {
    [
        F(PADDING_TOP, value),
        F(PADDING_RIGHT, value),
        F(PADDING_BOTTOM, value),
        F(PADDING_LEFT, value),
    ]
}

/// A layout box: CSS mode when `css` is Some, translated otherwise.
#[derive(Clone, Default)]
struct B {
    name: &'static str,
    css: Option<Vec<P>>,
    /// Translated mode: `LayoutComponent` keys (width, height, fractions).
    component: Vec<P>,
    /// Translated mode: `LayoutComponentStyle` keys.
    style: Vec<P>,
    children: Vec<B>,
    /// Drawing content: rectangles, each a Shape with one Rectangle path.
    content: Vec<Rect>,
    /// A ScrollConstraint on this box (its content box), with its direction
    /// (0 horizontal, 1 vertical); its parent is the viewport.
    scroll: Option<u64>,
}

/// A rectangle shape named `name` (its path is `name` + " path"), authored
/// at `size`, directly in its box or inside a plain group.
#[derive(Clone, Copy)]
struct Rect {
    name: &'static str,
    size: (f32, f32),
    grouped: bool,
}

fn css(name: &'static str, props: impl IntoIterator<Item = P>, children: Vec<B>) -> B {
    B {
        name,
        css: Some(props.into_iter().collect()),
        children,
        ..B::default()
    }
}

/// Every unit a translated style leaves undefined is points, as the Rive
/// editor writes them.
fn point_units() -> Vec<P> {
    [
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
    ]
    .into_iter()
    .map(|key| U(key, YGUnit::Point as u64))
    .collect()
}

struct Scene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
}

enum Entry<'a> {
    Box(&'a B, u64),
    Record(&'a [P], u64),
    Group(u64),
    Shape(&'a Rect, u64),
    Path(&'a Rect, u64),
    Scroll(u64, u64),
}

fn write(riv: &mut Riv, props: &[P]) {
    for prop in props {
        match *prop {
            U(key, value) => riv.uint(key, value),
            F(key, value) => riv.float(key, value),
        };
    }
}

/// Imports one artboard of `size` holding `root`, or None when the file is
/// refused. `artboard_css` puts a record on the artboard itself.
fn try_scene(size: (f32, f32), artboard_css: Option<Vec<P>>, root: B) -> Option<Scene> {
    fn walk<'a>(node: &'a B, parent: u64, entries: &mut Vec<Entry<'a>>) {
        let index = entries.len() as u64 + 1;
        entries.push(Entry::Box(node, parent));
        if let Some(props) = &node.css {
            entries.push(Entry::Record(props, index));
        }
        if let Some(direction) = node.scroll {
            entries.push(Entry::Scroll(index, direction));
        }
        for rect in &node.content {
            let mut parent = index;
            if rect.grouped {
                entries.push(Entry::Group(index));
                parent = entries.len() as u64;
            }
            entries.push(Entry::Shape(rect, parent));
            entries.push(Entry::Path(rect, entries.len() as u64));
        }
        for child in &node.children {
            walk(child, index, entries);
        }
    }
    let mut entries = Vec::new();
    if let Some(props) = &artboard_css {
        entries.push(Entry::Record(props, 0));
    }
    walk(&root, 0, &mut entries);
    // Styles follow every other object: the artboard's, then one per box.
    let first_style = entries.len() as u64 + 1;
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Artboard")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, size.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, size.1)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style)
        .end();
    let mut boxes = Vec::new();
    for entry in &entries {
        match entry {
            Entry::Box(node, parent) => {
                riv.object(LayoutComponentBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, node.name)
                    .uint(
                        LayoutComponentBase::STYLE_ID_PROPERTY_KEY,
                        first_style + 1 + boxes.len() as u64,
                    );
                write(&mut riv, &node.component);
                riv.end();
                boxes.push(*node);
            }
            Entry::Record(props, parent) => {
                riv.object(LAYOUT_CSS_STYLE)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent);
                write(&mut riv, props);
                riv.end();
            }
            Entry::Scroll(parent, direction) => {
                use crate::source::generated::constraints::draggable_constraint_base::DraggableConstraintBase;
                riv.object(crate::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY, *direction)
                    .end();
            }
            Entry::Group(parent) => {
                riv.object(NodeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::Shape(rect, parent) => {
                riv.object(ShapeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, rect.name)
                    .end();
            }
            Entry::Path(rect, parent) => {
                riv.object(RectangleBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(
                        ComponentBase::NAME_PROPERTY_KEY,
                        &format!("{} path", rect.name),
                    )
                    .float(ParametricPathBase::WIDTH_PROPERTY_KEY, rect.size.0)
                    .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, rect.size.1)
                    .end();
            }
        }
    }
    let style = |riv: &mut Riv, parent: u64, props: &[P]| {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        write(riv, &point_units());
        write(riv, props);
        riv.end();
    };
    style(&mut riv, 0, &[]);
    let mut index = 0u64;
    for entry in &entries {
        index += 1;
        if let Entry::Box(node, _) = entry {
            style(&mut riv, index, &node.style);
        }
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )?;
    let artboard = file.with_file(File::artboard_default)?;
    let scene = Scene {
        _file: file,
        artboard,
    };
    scene.advance(3);
    Some(scene)
}

fn scene(size: (f32, f32), root: B) -> Scene {
    try_scene(size, None, root).expect("the scene imports")
}

impl Scene {
    fn named(&self, name: &str) -> CoreHandle {
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<LayoutComponent>())
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
    fn advance(&self, frames: usize) {
        for _ in 0..frames {
            self.artboard.advance_default(FRAME);
        }
    }
    /// A box's solved left, top, width and height, relative to its parent.
    fn bounds(&self, name: &str) -> [f32; 4] {
        let bounds = self
            .named(name)
            .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds())
            .expect(name);
        [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
    }
    /// A rectangle's current path size, which layout writes when it
    /// resizes the shape.
    fn path_size(&self, name: &str) -> (f32, f32) {
        let path = format!("{name} path");
        self.artboard
            .with_artboard(|artboard| {
                artboard.find_all_handles::<crate::source::shapes::rectangle::Rectangle>()
            })
            .into_iter()
            .find_map(|handle| {
                handle
                    .with(|object| {
                        let path_object = object.as_parametric_path()?;
                        (object.as_component()?.base.name() == path)
                            .then(|| (path_object.width(), path_object.height()))
                    })
                    .flatten()
            })
            .unwrap_or_else(|| panic!("{path}"))
    }
    /// The scroll constraint's offset range: (max x, max y), both <= 0.
    fn max_offsets(&self) -> (f32, f32) {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        let scroll = self
            .artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("a scroll constraint");
        scroll
            .with_downcast::<ScrollConstraint, _>(|scroll| {
                (scroll.max_offset_x(), scroll.max_offset_y())
            })
            .unwrap()
    }
    /// A box's background and clip paths, in its own and world space.
    fn paths(
        &self,
        name: &str,
    ) -> (
        Vec<crate::source::math::vec2d::Vec2D>,
        Option<crate::source::math::aabb::Aabb>,
        bool,
    ) {
        self.named(name)
            .with_downcast::<LayoutComponent, _>(|layout| {
                let paths = layout.render_paths_handle().expect("render paths");
                let paths = paths.borrow();
                let clips = layout.clips();
                (
                    paths.background.points().to_vec(),
                    clips.then(|| paths.clip.raw_path().bounds()),
                    clips,
                )
            })
            .unwrap()
    }
    fn record(&self, name: &str) -> CoreHandle {
        let owner = self.named(name);
        owner
            .with(|object| {
                object.as_container_component().and_then(|container| {
                    container
                        .children()
                        .iter()
                        .find(|child| child.is_type_of(LAYOUT_CSS_STYLE))
                        .cloned()
                })
            })
            .flatten()
            .unwrap_or_else(|| panic!("{name} has a record"))
    }
}

/// `display: flex` with padding and a gap: the second item grows into what
/// the first leaves.
#[test]
fn css_flex_row_lays_out_by_its_record() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [
                [U(DISPLAY, 1), U(BOX_SIZING, 1)].as_slice(),
                &width(400.0),
                &height(100.0),
                &padding(10.0),
                &[F(COLUMN_GAP, 20.0)],
            ]
            .concat(),
            vec![
                css("A", [width(50.0), height(20.0)].concat(), vec![]),
                css(
                    "B",
                    [[F(FLEX_GROW, 1.0)].as_slice(), &height(20.0)].concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("Row"), [0.0, 0.0, 400.0, 100.0]);
    // 400 - 2 * 10 padding - 50 - 20 gap = 310.
    assert_eq!(f.bounds("A"), [10.0, 10.0, 50.0, 20.0]);
    assert_eq!(f.bounds("B"), [80.0, 10.0, 310.0, 20.0]);
}

/// `box-sizing` is content-box unless the record says border-box.
#[test]
fn css_width_excludes_padding_and_border_by_default() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Box",
            [
                [U(DISPLAY, FLEX as u64)].as_slice(),
                &width(100.0),
                &height(50.0),
                &padding(10.0),
                &[
                    F(BORDER_TOP, 2.0),
                    F(BORDER_RIGHT, 2.0),
                    F(BORDER_BOTTOM, 2.0),
                    F(BORDER_LEFT, 2.0),
                ],
            ]
            .concat(),
            vec![css("Child", [width(10.0), height(10.0)].concat(), vec![])],
        ),
    );
    // 100 + 2 * (10 + 2) by 50 + 2 * (10 + 2).
    assert_eq!(f.bounds("Box"), [0.0, 0.0, 124.0, 74.0]);
    assert_eq!(f.bounds("Child"), [12.0, 12.0, 10.0, 10.0]);
}

/// `display: block` (the record's initial display): children stack at the
/// full width, and adjacent vertical margins collapse.
#[test]
fn css_block_flow_stacks_and_collapses_margins() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Column",
            width(200.0),
            vec![
                css(
                    "A",
                    [[F(MARGIN_BOTTOM, 10.0)].as_slice(), &height(30.0)].concat(),
                    vec![],
                ),
                css(
                    "B",
                    [[F(MARGIN_TOP, 20.0)].as_slice(), &height(40.0)].concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("A"), [0.0, 0.0, 200.0, 30.0]);
    // max(10, 20) between them.
    assert_eq!(f.bounds("B"), [0.0, 50.0, 200.0, 40.0]);
    assert_eq!(f.bounds("Column"), [0.0, 0.0, 200.0, 90.0]);
}

/// flex-shrink comes from the record: 0 keeps its width, the initial 1
/// gives up the overflow.
#[test]
fn css_flex_shrink_comes_from_the_record() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![
                css(
                    "Keep",
                    [[F(FLEX_SHRINK, 0.0)].as_slice(), &width(200.0)].concat(),
                    vec![],
                ),
                css("Give", width(200.0), vec![]),
            ],
        ),
    );
    assert_eq!(f.bounds("Keep")[2], 200.0);
    assert_eq!(f.bounds("Give"), [200.0, 0.0, 100.0, 40.0]);
}

/// Percentages resolve against the parent, and an absolute box is placed by
/// its insets inside its positioned parent.
#[test]
fn css_percentages_and_absolute_insets() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(DISPLAY, 1), U(POSITION, 1)].as_slice(),
                &width(400.0),
                &height(200.0),
            ]
            .concat(),
            vec![
                css(
                    "Half",
                    [
                        U(WIDTH_KIND, 1),
                        F(WIDTH_PERCENT, 50.0),
                        U(HEIGHT_KIND, 1),
                        F(HEIGHT, 10.0),
                    ],
                    vec![],
                ),
                css(
                    "Pinned",
                    [
                        [
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 10.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 20.0),
                        ]
                        .as_slice(),
                        &width(30.0),
                        &height(30.0),
                    ]
                    .concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("Half"), [0.0, 0.0, 200.0, 10.0]);
    assert_eq!(f.bounds("Pinned"), [10.0, 20.0, 30.0, 30.0]);
}

/// `display: none` takes the box out of flow and hides it.
#[test]
fn css_display_none_hides_the_box() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![
                css(
                    "Gone",
                    [[U(DISPLAY, NONE as u64)].as_slice(), &width(100.0)].concat(),
                    vec![],
                ),
                css("Next", width(50.0), vec![]),
            ],
        ),
    );
    assert_eq!(f.bounds("Next")[0], 0.0);
    let hidden = f
        .named("Gone")
        .with_downcast::<LayoutComponent, _>(LayoutComponent::is_collapsed)
        .unwrap();
    assert!(hidden);
}

/// A record on the artboard sets its container properties; the artboard
/// keeps its own size.
#[test]
fn css_artboard_record_sets_container_properties() {
    let f = try_scene(
        (400.0, 300.0),
        // Its width is an item property, so the artboard ignores it.
        Some(
            [
                [U(DISPLAY, 1), U(WIDTH_KIND, LENGTH as u64), F(WIDTH, 10.0)].as_slice(),
                &padding(15.0),
            ]
            .concat(),
        ),
        css("Child", [width(50.0), height(20.0)].concat(), vec![]),
    )
    .expect("the scene imports");
    assert_eq!(f.bounds("Child"), [15.0, 15.0, 50.0, 20.0]);
    let artboard = f
        .artboard
        .with_artboard(|artboard| artboard.layout_bounds());
    assert_eq!((artboard.width(), artboard.height()), (400.0, 300.0));
}

/// A record value written at run time, as animation and binding write it,
/// lays the box out again.
#[test]
fn css_record_writes_relayout() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![css("A", width(50.0), vec![])],
        ),
    );
    assert_eq!(f.bounds("A")[2], 50.0);
    assert!(CoreRegistry::set_double_handle(
        &f.record("A"),
        WIDTH.into(),
        120.0
    ));
    assert_eq!(
        CoreRegistry::get_double_handle(&f.record("A"), WIDTH.into()),
        Some(120.0)
    );
    f.advance(2);
    assert_eq!(f.bounds("A")[2], 120.0);
    assert!(CoreRegistry::set_uint_handle(
        &f.record("A"),
        DISPLAY.into(),
        3
    ));
    f.advance(2);
    assert!(
        f.named("A")
            .with_downcast::<LayoutComponent, _>(LayoutComponent::is_collapsed)
            .unwrap()
    );
}

/// Refused at load (docs/CSS_LAYOUT.md, 1.4): the import fails.
#[test]
fn css_refused_forms_fail_the_import() {
    let refused =
        |props: Vec<P>| try_scene((400.0, 300.0), None, css("Box", props, vec![])).is_none();
    // A mixed calc(): 10px + 50%.
    assert!(refused(vec![
        U(WIDTH_KIND, 1),
        F(WIDTH, 10.0),
        F(WIDTH_PERCENT, 50.0)
    ]));
    // display 9 is outside 0 to 4.
    assert!(refused(vec![U(DISPLAY, 9)]));
    // A size kind outside 0 to 5.
    assert!(refused(vec![U(WIDTH_KIND, 6)]));
    // Mixed parts are not read for an auto width, so they are not refused.
    assert!(!refused(vec![F(WIDTH, 10.0), F(WIDTH_PERCENT, 50.0)]));
    assert!(!refused(vec![U(WIDTH_KIND, 1), F(WIDTH, 10.0)]));
    // Two records on one box.
    assert!(try_two_records().is_none());
}

/// Two records under one box.
fn try_two_records() -> Option<Scene> {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
        .end();
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5)
        .end();
    for _ in 0..2 {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .end();
    }
    for parent in [0, 1] {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .end();
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )?;
    let artboard = file.with_file(File::artboard_default)?;
    Some(Scene {
        _file: file,
        artboard,
    })
}

/// Translated mode: Fill along the main axis is `flex: <fraction> 1 0%`, so
/// Fill siblings share the row by fraction whatever their content. Rive's
/// own Fill sized each to its content first (here 180 and 120).
#[test]
fn translated_fill_shares_the_row_by_fraction() {
    let fill = |name: &'static str, children: Vec<B>| B {
        name,
        component: vec![F(LayoutComponentBase::FRACTIONAL_WIDTH_PROPERTY_KEY, 1.0)],
        style: vec![
            U(
                LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Fill as u64,
            ),
            U(
                LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Fill as u64,
            ),
        ],
        children,
        ..B::default()
    };
    let content = B {
        name: "Content",
        component: vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 60.0),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 20.0),
        ],
        ..B::default()
    };
    let f = scene(
        (300.0, 100.0),
        B {
            name: "Row",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 300.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0),
            ],
            style: vec![U(
                LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                YGFlexDirection::Row as u64,
            )],
            children: vec![fill("Wide", vec![content]), fill("Empty", vec![])],
            ..B::default()
        },
    );
    assert_eq!(f.bounds("Wide")[2], 150.0);
    assert_eq!(f.bounds("Empty"), [150.0, 0.0, 150.0, 100.0]);
}

/// CSS mode: a shape is content at its natural size, as an `<svg width>` is.
/// Its box hugs it, keeps it as its automatic minimum, and never resizes it.
#[test]
fn css_shape_is_content_at_its_natural_size() {
    let rect = |name| Rect {
        name,
        size: (80.0, 30.0),
        grouped: false,
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(100.0), &height(40.0)].concat(),
            vec![
                B {
                    content: vec![rect("Icon")],
                    ..css("Hug", [], vec![])
                },
                css("Wide", width(100.0), vec![]),
            ],
        ),
    );
    // Hug's basis is 80 and its minimum 80; Wide gives up the overflow.
    assert_eq!(f.bounds("Hug"), [0.0, 0.0, 80.0, 40.0]);
    assert_eq!(f.bounds("Wide"), [80.0, 0.0, 20.0, 40.0]);
    assert_eq!(f.path_size("Icon"), (80.0, 30.0));

    // A box wider than its shape leaves the shape at its own size.
    let f = scene(
        (400.0, 300.0),
        B {
            content: vec![rect("Icon")],
            ..css("Box", [width(200.0), height(100.0)].concat(), vec![])
        },
    );
    assert_eq!(f.bounds("Box"), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(f.path_size("Icon"), (80.0, 30.0));
}

/// Translated mode: a shape the box resizes is the box's paint, so it adds
/// nothing to the box's automatic minimum. The Fixed box shrinks with its
/// sibling in proportion to their bases (80 and 100 sharing an 80 overflow).
/// A grouped shape is content and keeps the minimum.
#[test]
fn translated_paint_shape_adds_no_minimum() {
    let row = |grouped: bool| {
        let fixed = |name: &'static str, width: f32, content: Vec<Rect>| B {
            name,
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, width),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
            ],
            content,
            ..B::default()
        };
        scene(
            (400.0, 300.0),
            B {
                name: "Row",
                component: vec![
                    F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                    F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
                ],
                style: vec![U(
                    LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                    YGFlexDirection::Row as u64,
                )],
                children: vec![
                    fixed(
                        "Painted",
                        80.0,
                        vec![Rect {
                            name: "Background",
                            size: (80.0, 40.0),
                            grouped,
                        }],
                    ),
                    fixed("Sibling", 100.0, vec![]),
                ],
                ..B::default()
            },
        )
    };
    let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
    let f = row(false);
    let painted = f.bounds("Painted")[2];
    assert!(close(painted, 80.0 - 80.0 * 80.0 / 180.0), "{painted}");
    assert!(close(f.bounds("Sibling")[2], 100.0 - 80.0 * 100.0 / 180.0));
    // Layout resized the background to its box.
    assert!(close(f.path_size("Background").0, painted));

    let f = row(true);
    assert_eq!(f.bounds("Painted")[2], 80.0);
    assert_eq!(f.bounds("Sibling")[2], 20.0);
}

/// CSS mode clips by `overflow`, at the padding box, and not by Rive's clip
/// (on by default, but CSS mode ignores it). The background has CSS radii.
#[test]
fn css_overflow_clips_at_the_padding_box() {
    const OVERFLOW_X: u16 = 60025;
    const OVERFLOW_Y: u16 = 60026;
    const RADIUS_TL_PERCENT: u16 = 60101;
    let border = [
        F(BORDER_TOP, 4.0),
        F(BORDER_RIGHT, 4.0),
        F(BORDER_BOTTOM, 4.0),
        F(BORDER_LEFT, 4.0),
    ];
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [[U(DISPLAY, 1)].as_slice(), &width(400.0), &height(300.0)].concat(),
            vec![
                css(
                    "Clipped",
                    [
                        [U(BOX_SIZING, 1), U(OVERFLOW_Y, 2)].as_slice(),
                        &width(100.0),
                        &height(60.0),
                        &border,
                    ]
                    .concat(),
                    vec![],
                ),
                css(
                    "Pill",
                    [
                        [
                            U(OVERFLOW_X, 1),
                            U(OVERFLOW_Y, 1),
                            F(RADIUS_TL_PERCENT, 50.0),
                        ]
                        .as_slice(),
                        &width(200.0),
                        &height(100.0),
                    ]
                    .concat(),
                    vec![],
                ),
                css("Visible", [width(10.0), height(10.0)].concat(), vec![]),
            ],
        ),
    );
    // overflow-y hidden makes overflow-x auto: both axes clip, inside the
    // 4-point border.
    let (_, clip, clips) = f.paths("Clipped");
    assert!(clips);
    let clip = clip.unwrap();
    assert_eq!(
        [clip.left(), clip.top(), clip.right(), clip.bottom()],
        [4.0, 4.0, 96.0, 56.0]
    );
    // 50% of 200 by 100 at the top left: a 100 by 50 ellipse, so the path
    // starts 100 along the top edge.
    let (background, _, clips) = f.paths("Pill");
    assert!(clips);
    assert_eq!(
        background[0],
        crate::source::math::vec2d::Vec2D::new(100.0, 0.0)
    );
    let visible = f
        .named("Visible")
        .with_downcast::<LayoutComponent, _>(LayoutComponent::clips)
        .unwrap();
    assert!(!visible, "Rive's default clip is ignored in CSS mode");
}

/// A CSS viewport's scroll range is its scrollable overflow less its padding
/// box: a 120-point block list in an 80-point view stops at 40.
#[test]
fn css_scroll_range_comes_from_scrollable_overflow() {
    const OVERFLOW_Y: u16 = 60026;
    let item = |name| css(name, height(60.0), vec![]);
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [[U(OVERFLOW_Y, 3)].as_slice(), &width(200.0), &height(80.0)].concat(),
            vec![B {
                scroll: Some(1),
                ..css("List", [], vec![item("One"), item("Two")])
            }],
        ),
    );
    assert_eq!(f.bounds("List"), [0.0, 0.0, 200.0, 120.0]);
    assert_eq!(f.max_offsets().1, -40.0);
}

/// CSS `position`: a static box is not a containing block, so an absolute
/// grandchild is placed against the positioned frame, and its bounds stay
/// relative to its own parent.
#[test]
fn css_absolute_box_skips_a_static_parent() {
    const MARGIN_LEFT: u16 = 60071;
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [[U(POSITION, 1)].as_slice(), &width(300.0), &height(200.0)].concat(),
            vec![css(
                "Static",
                [
                    [F(MARGIN_LEFT, 50.0)].as_slice(),
                    &width(100.0),
                    &height(40.0),
                ]
                .concat(),
                vec![css(
                    "Pinned",
                    [
                        [
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 10.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 10.0),
                        ]
                        .as_slice(),
                        &width(30.0),
                        &height(30.0),
                    ]
                    .concat(),
                    vec![],
                )],
            )],
        ),
    );
    assert_eq!(f.bounds("Static"), [50.0, 0.0, 100.0, 40.0]);
    // (10, 10) in the frame is (-40, 10) in the static box.
    assert_eq!(f.bounds("Pinned"), [-40.0, 10.0, 30.0, 30.0]);
}
