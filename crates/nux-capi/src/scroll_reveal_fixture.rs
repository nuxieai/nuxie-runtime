//! Pages whose buttons sit in a CSS scroll box, for a host revealing a
//! node scrolled out of view and moving a region's content.
//!
//! The file has no view model and no binds: every record is written by hand.
//! Each page ("Page", artboard 0, 320 by 400) is a CSS block column whose
//! first child, "Box", is a scroll viewport 200 by 100 at the page's top
//! left, as the compiler writes one (relative, its overflow scrolling),
//! scrolling a "Content" box 300 points long. Each labeled box is a button:
//! a box with a `SemanticData` of role button and that label, whose
//! semantic tap fires an event. A page's scroll constraints share the fixture's one physics
//! object, upstream's elastic or clamped, or have none.
//!
//! The [`Page`] variants place the buttons; each lists its own positions.
//! Artboard 1, "Row" (200 by 40), is the keyed list's row, a [`ROW_BUTTON`]
//! box filling it; artboard 2, "Used" (200 by 40), is the used component, a
//! [`USED_BUTTON`] box filling it.
#![allow(dead_code)]
use nuxie_schema::definition_by_name;

/// `DrawableFlag::OPAQUE`: a box whose hit blocks what is behind it.
const OPAQUE: u64 = 1 << 3;
/// `ListenerType::SemanticAction`.
const SEMANTIC_ACTION: u64 = 16;
/// `SemanticActionType::Tap`.
const TAP: u64 = 0;

/// One property value as the file writes it.
#[derive(Clone, Copy)]
enum V<'a> {
    U(u64),
    F(f32),
    S(&'a str),
}

/// The key of `kind`'s property `name`, its own or an ancestor's.
fn key(kind: &str, name: &str) -> u16 {
    let definition = definition_by_name(kind).unwrap_or_else(|| panic!("type {kind}"));
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|definition| definition.properties)
        .find(|property| property.name == name)
        .unwrap_or_else(|| panic!("{kind}.{name}"))
        .key
        .int
}

/// One object of `kind` with `props`, then its terminator.
fn obj(b: &mut Vec<u8>, kind: &str, props: &[(&str, V<'_>)]) {
    let kind_key = definition_by_name(kind)
        .unwrap_or_else(|| panic!("type {kind}"))
        .type_key
        .int;
    var(b, kind_key.into());
    for (name, value) in props {
        var(b, key(kind, name).into());
        match value {
            V::U(n) => var(b, *n),
            V::F(v) => b.extend(v.to_le_bytes()),
            V::S(s) => {
                var(b, s.len() as u64);
                b.extend(s.as_bytes());
            }
        }
    }
    var(b, 0);
}

pub const TOP: &str = "Top";
pub const MIDDLE: &str = "Middle";
pub const BOTTOM: &str = "Bottom";
pub const DEEP: &str = "Deep";
pub const ROW_BUTTON: &str = "Row button";
pub const USED_BUTTON: &str = "Used button";
pub const START: &str = "Start";
pub const FAR: &str = "Far";

/// The viewport's size, and the content's length along the way it scrolls.
pub const VIEWPORT_WIDTH: f32 = 200.0;
pub const VIEWPORT_HEIGHT: f32 = 100.0;
pub const CONTENT_LENGTH: f32 = 300.0;

/// The keyed list's local id on the page, in [`Page::Hosted`].
pub const ROWS: usize = 17;

/// Where a page puts its buttons, each 100 by 40 unless it says otherwise,
/// with plain boxes as gaps between them, in "Content" from its top.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// [`TOP`] at 0, [`MIDDLE`] at 150, [`BOTTOM`] at 220 (its bottom edge
    /// at 260), then 40 points of gap.
    Buttons,
    /// [`TOP`] at 0, then at 150 an inner scroll viewport, "Inner", 200 by
    /// 60, scrolling "Inner content" 140 long: 100 points of gap, then
    /// [`DEEP`] at 100. Then 90 points of gap.
    Nested,
    /// [`TOP`] at 0, at 150 a used component of artboard 2 (200 by 40,
    /// [`USED_BUTTON`] filling it), and at 220 the keyed list of artboard 1
    /// rows ([`ROWS`]), each 40 tall with [`ROW_BUTTON`] filling it, so two
    /// rows end the content at 300.
    Hosted,
    /// A right-to-left viewport 100 wide and 40 tall scrolling sideways a
    /// row of three 100 point boxes: [`START`], a gap, and [`FAR`]. Its
    /// start is its right edge, so [`START`] shows and [`FAR`] is 200 points
    /// to the left of the viewport.
    Sideways,
}

/// One box of a page: a `LayoutComponent` and its CSS record, with a
/// button's semantic data when labeled, its scroll constraint when it is a
/// viewport's content, and its children.
enum Item {
    Box {
        name: &'static str,
        css: Vec<(&'static str, V<'static>)>,
        label: Option<&'static str>,
        /// The scroll direction of the constraint on this box: 0 sideways,
        /// 1 vertical.
        scroll: Option<u64>,
        children: Vec<Item>,
    },
    /// A used component of artboard 2, with its state machine.
    Used,
    /// A keyed list whose rows are artboard 1.
    Rows,
}

fn sized(width: Option<f32>, height: Option<f32>) -> Vec<(&'static str, V<'static>)> {
    use V::*;
    let mut props = Vec::new();
    if let Some(width) = width {
        props.extend([("widthKind", U(1)), ("width", F(width))]);
    }
    if let Some(height) = height {
        props.extend([("heightKind", U(1)), ("height", F(height))]);
    }
    props
}

fn button(label: &'static str, width: f32, height: f32) -> Item {
    Item::Box {
        name: label,
        css: sized(Some(width), Some(height)),
        label: Some(label),
        scroll: None,
        children: vec![],
    }
}

fn gap(name: &'static str, height: f32) -> Item {
    Item::Box {
        name,
        css: sized(Some(VIEWPORT_WIDTH), Some(height)),
        label: None,
        scroll: None,
        children: vec![],
    }
}

/// A vertical scroll viewport `name`, scrolling `content` over `children`.
fn viewport(name: &'static str, content: &'static str, height: f32, children: Vec<Item>) -> Item {
    use V::*;
    let mut css = sized(Some(VIEWPORT_WIDTH), Some(height));
    css.extend([("position", U(1)), ("overflowY", U(3))]);
    Item::Box {
        name,
        css,
        label: None,
        scroll: None,
        children: vec![Item::Box {
            name: content,
            css: vec![],
            label: None,
            scroll: Some(1),
            children,
        }],
    }
}

fn page_items(page: Page) -> Item {
    use V::*;
    match page {
        Page::Buttons => viewport(
            "Box",
            "Content",
            VIEWPORT_HEIGHT,
            vec![
                button(TOP, 100.0, 40.0),
                gap("Gap 1", 110.0),
                button(MIDDLE, 100.0, 40.0),
                gap("Gap 2", 30.0),
                button(BOTTOM, 100.0, 40.0),
                gap("Gap 3", 40.0),
            ],
        ),
        Page::Nested => viewport(
            "Box",
            "Content",
            VIEWPORT_HEIGHT,
            vec![
                button(TOP, 100.0, 40.0),
                gap("Gap 1", 110.0),
                viewport(
                    "Inner",
                    "Inner content",
                    60.0,
                    vec![gap("Inner gap", 100.0), button(DEEP, 100.0, 40.0)],
                ),
                gap("Gap 2", 90.0),
            ],
        ),
        Page::Hosted => viewport(
            "Box",
            "Content",
            VIEWPORT_HEIGHT,
            vec![
                button(TOP, 100.0, 40.0),
                gap("Gap 1", 110.0),
                Item::Used,
                gap("Gap 2", 30.0),
                Item::Rows,
            ],
        ),
        Page::Sideways => {
            let shrinks = |label: Option<&'static str>, name: &'static str| Item::Box {
                name,
                css: [vec![("flexShrink", F(0.0))], sized(Some(100.0), Some(40.0))].concat(),
                label,
                scroll: None,
                children: vec![],
            };
            Item::Box {
                name: "Box",
                css: [
                    vec![
                        ("display", U(1)),
                        ("direction", U(2)),
                        ("overflowX", U(3)),
                        ("position", U(1)),
                    ],
                    sized(Some(100.0), Some(40.0)),
                ]
                .concat(),
                label: None,
                scroll: None,
                children: vec![Item::Box {
                    name: "Content",
                    css: [
                        vec![("display", U(1)), ("flexShrink", F(0.0))],
                        sized(Some(CONTENT_LENGTH), Some(40.0)),
                    ]
                    .concat(),
                    label: None,
                    scroll: Some(0),
                    children: vec![
                        shrinks(Some(START), START),
                        shrinks(None, "Gap"),
                        shrinks(Some(FAR), FAR),
                    ],
                }],
            }
        }
    }
}

/// One artboard's objects, its local ids counted as they are written.
struct Board {
    b: Vec<u8>,
    count: usize,
    /// The first style's local id; styles are written last.
    style_base: usize,
    /// Each layout box with a style, the root first, in style order.
    styled: Vec<usize>,
    /// Each labeled box, whose semantic tap fires an event.
    buttons: Vec<usize>,
    /// Whether the file has a physics object, local id 0 of the backboard's.
    physics: bool,
}

impl Board {
    fn add(&mut self, kind: &str, props: &[(&str, V<'_>)]) -> usize {
        obj(&mut self.b, kind, props);
        self.count += 1;
        self.count - 1
    }

    fn next_style(&mut self, owner: usize) -> u64 {
        self.styled.push(owner);
        (self.style_base + self.styled.len() - 1) as u64
    }

    fn item(&mut self, parent: usize, item: &Item) {
        use V::*;
        match item {
            Item::Box {
                name,
                css,
                label,
                scroll,
                children,
            } => {
                let id = self.count;
                let style = self.next_style(id);
                let mut props = vec![
                    ("name", S(name)),
                    ("parentId", U(parent as u64)),
                    ("styleId", U(style)),
                ];
                if label.is_some() {
                    props.push(("drawableFlags", U(OPAQUE)));
                }
                self.add("LayoutComponent", &props);
                let mut record = vec![("parentId", U(id as u64))];
                record.extend(css.iter().copied());
                self.add("LayoutCssStyle", &record);
                if let Some(direction) = scroll {
                    let mut constraint = vec![
                        ("parentId", U(id as u64)),
                        ("directionValue", U(*direction)),
                        ("threshold", F(10.0)),
                    ];
                    if self.physics {
                        constraint.push(("physicsId", U(0)));
                    }
                    self.add("ScrollConstraint", &constraint);
                }
                if let Some(label) = label {
                    self.add(
                        "SemanticData",
                        &[
                            ("parentId", U(id as u64)),
                            ("role", U(1)),
                            ("label", S(label)),
                        ],
                    );
                    self.buttons.push(id);
                }
                for child in children {
                    self.item(id, child);
                }
            }
            Item::Used => {
                let id = self.add(
                    "NestedArtboardLayout",
                    &[
                        ("name", S("Used")),
                        ("parentId", U(parent as u64)),
                        ("artboardId", U(2)),
                    ],
                );
                let mut record = vec![("parentId", U(id as u64))];
                record.extend(sized(Some(VIEWPORT_WIDTH), Some(40.0)));
                self.add("LayoutCssStyle", &record);
                self.add(
                    "NestedStateMachine",
                    &[("parentId", U(id as u64)), ("animationId", U(0))],
                );
            }
            Item::Rows => {
                let id = self.add(
                    "ArtboardComponentList",
                    &[
                        ("name", S("Rows")),
                        ("parentId", U(parent as u64)),
                        ("rowArtboardId", U(1)),
                    ],
                );
                assert_eq!(id, ROWS, "the list's local id");
            }
        }
    }
}

/// An artboard of `width` by `height` holding `root`'s children, with a
/// state machine whose listeners fire `pressed` on each button's semantic
/// tap.
fn artboard(
    name: &str,
    width: f32,
    height: f32,
    children: &[Item],
    physics: bool,
    style_base: usize,
) -> Board {
    use V::*;
    let mut board = Board {
        b: Vec::new(),
        count: 0,
        style_base,
        styled: Vec::new(),
        buttons: Vec::new(),
        physics,
    };
    let style = style_base as u64;
    board.styled.push(0);
    board.add(
        "Artboard",
        &[
            ("name", S(name)),
            ("width", F(width)),
            ("height", F(height)),
            ("styleId", U(style)),
        ],
    );
    board.add("LayoutCssStyle", &[("parentId", U(0))]);
    for child in children {
        board.item(0, child);
    }
    board
}

fn write_artboard(
    b: &mut Vec<u8>,
    name: &str,
    width: f32,
    height: f32,
    children: &[Item],
    physics: bool,
) {
    use V::*;
    // The styles follow every other object; count those first.
    let objects = artboard(name, width, height, children, physics, 0).count + 1;
    let mut board = artboard(name, width, height, children, physics, objects);
    let event = board.add("Event", &[("name", S("pressed")), ("parentId", U(0))]);
    for owner in board.styled.clone() {
        board.add("LayoutComponentStyle", &[("parentId", U(owner as u64))]);
    }
    obj(&mut board.b, "StateMachine", &[("name", S("machine"))]);
    for button in board.buttons.clone() {
        obj(
            &mut board.b,
            "StateMachineListenerSingle",
            &[
                ("targetId", U(button as u64)),
                ("listenerTypeValue", U(SEMANTIC_ACTION)),
            ],
        );
        obj(&mut board.b, "ListenerInputTypeSemantic", &[]);
        obj(&mut board.b, "SemanticInput", &[("actionType", U(TAP))]);
        obj(
            &mut board.b,
            "ListenerFireEvent",
            &[("eventId", U(event as u64))],
        );
    }
    obj(&mut board.b, "StateMachineLayer", &[("name", S("layer"))]);
    obj(&mut board.b, "EntryState", &[]);
    obj(&mut board.b, "AnyState", &[]);
    obj(&mut board.b, "ExitState", &[]);
    b.extend(board.b);
}

fn var(b: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
}

/// The scroll physics of a fixture's scroll constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Physics {
    None,
    /// Animates to a target (`ElasticScrollPhysics`).
    Elastic,
    /// Has no `scrollToPosition` of its own (`ClampedScrollPhysics`).
    Clamped,
}

/// The file of `page`, its scroll constraints with `physics`.
pub fn scroll_reveal_fixture(page: Page, kind: Physics) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    match kind {
        Physics::None => {}
        Physics::Elastic => obj(&mut b, "ElasticScrollPhysics", &[]),
        Physics::Clamped => obj(&mut b, "ClampedScrollPhysics", &[]),
    }
    let physics = kind != Physics::None;
    write_artboard(&mut b, "Page", 320.0, 400.0, &[page_items(page)], physics);
    write_artboard(
        &mut b,
        "Row",
        VIEWPORT_WIDTH,
        40.0,
        &[button(ROW_BUTTON, VIEWPORT_WIDTH, 40.0)],
        physics,
    );
    write_artboard(
        &mut b,
        "Used",
        VIEWPORT_WIDTH,
        40.0,
        &[button(USED_BUTTON, VIEWPORT_WIDTH, 40.0)],
        physics,
    );
    b
}
