//! The layout hold of an element flying in place: css_layout/flight_hold.rs
//! and docs/live-moves.md, Flying in place.
//!
//! The matched element "E" stays where it was authored. On the upper Screen
//! it sits in a flex row between N1 and N2; on the lower Screen it sits in a
//! horizontal scroll view, scrolled so it is half out of view, between M1
//! and M2. E holds a child that stretches ("Fill", flex-grow 1 in a column)
//! and a text that wraps ("Label", 16 point words about 197 points wide).
//!
//! Oracle: the same Screen loaded fresh with E authored at the flight size.
//! On every frame of a step, forward, Back and cancelled halfway, with the
//! fresh-solve checker on:
//! - every box outside E rests where it rested;
//! - E's inside is laid out exactly as in the oracle;
//! - the tree holds exactly one placeholder (and its baseline marker);
//! - after the step, the resting tree comes back.
use super::*;
use crate::css_layout::flight_hold::{begin, end, resting_box, set_size};
use crate::source::layout::layout_participant::LayoutParticipant;
use crate::source::layout_component::{Layout, capture_layout_verify};

const SCROLL_OFFSET_X: u16 = 759;
const ALIGN_ITEMS: u16 = 61013;
const FLEX_DIRECTION: u16 = 61006;
const MIN_HEIGHT_KIND: u16 = 61030;
const INSET_RIGHT_KIND: u16 = 61054;
const INSET_RIGHT: u16 = 61055;
const INSET_BOTTOM_KIND: u16 = 61057;
const INSET_BOTTOM: u16 = 61058;

/// E's inside: Fill and Label.
fn inside() -> Vec<B> {
    vec![
        css("Fill", [F(FLEX_GROW, 1.0), U(MIN_HEIGHT_KIND, 1)], vec![]),
        B {
            link: Some("tap"),
            ..css("Label", [], vec![])
        },
    ]
}

/// E's own properties: a border-box flex column with 10 points of padding.
fn element_props() -> Vec<P> {
    [
        [
            U(DISPLAY, 1),
            U(FLEX_DIRECTION, 2),
            U(BOX_SIZING, 1),
            F(FLEX_SHRINK, 0.0),
        ]
        .as_slice(),
        &padding(10.0),
    ]
    .concat()
}

/// E at `size`, with `extra` properties, holding `children`.
fn element_with(size: (f32, f32), extra: &[P], children: Vec<B>) -> B {
    css(
        "E",
        [
            element_props().as_slice(),
            &width(size.0),
            &height(size.1),
            extra,
        ]
        .concat(),
        children,
    )
}

fn element(size: (f32, f32)) -> B {
    element_with(size, &[], inside())
}

fn sized(name: &'static str, w: f32, h: f32) -> B {
    css(
        name,
        [[F(FLEX_SHRINK, 0.0)].as_slice(), &width(w), &height(h)].concat(),
        vec![],
    )
}

/// The upper Screen: E in a flex row between N1 and N2.
fn upper(e: (f32, f32)) -> Scene {
    upper_with(element(e))
}

fn upper_with(e: B) -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(DISPLAY, 1), U(ALIGN_ITEMS, 2), F(COLUMN_GAP, 10.0)].as_slice(),
                &width(400.0),
                &height(300.0),
            ]
            .concat(),
            vec![sized("N1", 80.0, 100.0), e, sized("N2", 80.0, 100.0)],
        ),
    )
}

/// The lower Screen: E in a horizontal scroll view 200 wide, scrolled 200
/// along, so E (120 to 280 in its strip) shows from -80 to 80: half out.
fn lower(e: (f32, f32)) -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [width(400.0), height(300.0)].concat(),
            vec![css(
                "Viewport",
                [
                    [U(OVERFLOW_X, 3), U(OVERFLOW_Y, 2)].as_slice(),
                    &width(200.0),
                    &height(150.0),
                ]
                .concat(),
                vec![B {
                    scroll: Some(0),
                    scroll_properties: vec![F(SCROLL_OFFSET_X, -200.0)],
                    ..css(
                        "Strip",
                        [U(DISPLAY, 1), U(ALIGN_ITEMS, 2)],
                        vec![
                            sized("M1", 120.0, 100.0),
                            element(e),
                            sized("M2", 120.0, 100.0),
                        ],
                    )
                }],
            )],
        ),
    )
}

const UPPER_NAMES: [&str; 6] = ["Frame", "N1", "E", "N2", "Fill", "Label"];
const LOWER_NAMES: [&str; 8] = [
    "Frame", "Viewport", "Strip", "M1", "E", "M2", "Fill", "Label",
];
const INSIDE: [&str; 2] = ["Fill", "Label"];
const LOWER_SIZE: (f32, f32) = (160.0, 120.0);
const UPPER_SIZE: (f32, f32) = (120.0, 100.0);

fn boxes(scene: &Scene, names: &[&str]) -> Vec<(String, [f32; 4])> {
    names
        .iter()
        .map(|name| (name.to_string(), scene.bounds(name)))
        .collect()
}

fn lerp(a: (f32, f32), b: (f32, f32), p: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * p, a.1 + (b.1 - a.1) * p)
}

/// (nodes, reachable nodes, placeholders, baseline markers).
fn counts(scene: &Scene) -> (usize, usize, usize, usize) {
    LayoutComponent::layout_tree_counts(&scene.artboard.core_handle()).expect("a layout root")
}

fn dirty(scene: &Scene, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|name| {
            scene
                .named(name)
                .with_downcast::<LayoutComponent, _>(|l| l.layout_data.is_dirty())
                .unwrap()
        })
        .map(|name| name.to_string())
        .collect()
}

/// The draw position a box's own paint reaches: its world translation.
fn world(scene: &Scene, name: &str) -> (f32, f32) {
    scene
        .named(name)
        .with(|object| {
            let world = object
                .as_world_transform_component()
                .expect("a transform")
                .world_transform();
            (world.tx(), world.ty())
        })
        .unwrap()
}

fn rect(layout: Layout) -> [f32; 4] {
    [layout.left(), layout.top(), layout.width(), layout.height()]
}

/// A Screen to fly E in: made with E authored at a size, its boxes, and E's
/// own resting size.
struct Screen<'a> {
    make: &'a dyn Fn((f32, f32)) -> Scene,
    names: &'a [&'a str],
    /// E's inside: compared with the oracle, not with the rest.
    inside: &'a [&'a str],
    own: (f32, f32),
}

fn upper_screen() -> Screen<'static> {
    Screen {
        make: &upper,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    }
}

fn lower_screen() -> Screen<'static> {
    Screen {
        make: &lower,
        names: &LOWER_NAMES,
        inside: &INSIDE,
        own: LOWER_SIZE,
    }
}

/// Runs `body` with the fresh-solve checker on and fails on any mismatch.
fn checked<T>(body: impl FnOnce() -> T) -> T {
    let mut out = None;
    let mismatches = capture_layout_verify(|| out = Some(body()));
    assert_eq!(mismatches, Vec::<String>::new(), "the fresh-solve checker");
    out.unwrap()
}

/// E flown from `from` to `to` over 20 frames, cut at `stop` (1 lands;
/// less cancels there). Checks every frame against the oracle and returns
/// the number of frames checked.
fn fly(screen: &Screen, from: (f32, f32), to: (f32, f32), stop: f32) -> usize {
    checked(|| fly_unchecked(screen, from, to, stop))
}

fn fly_unchecked(screen: &Screen, from: (f32, f32), to: (f32, f32), stop: f32) -> usize {
    let scene = (screen.make)(screen.own);
    scene.advance(2);
    let rest = boxes(&scene, screen.names);
    let rest_world: Vec<_> = screen.names.iter().map(|n| world(&scene, n)).collect();
    let rest_counts = counts(&scene);
    let e = scene.named("E");
    let e_rest = scene.bounds("E");
    assert!(begin(&e, from));
    assert!(!begin(&e, from), "one hold at a time");
    let mut frames = 0;
    for step in 0..=20 {
        let p = step as f32 / 20.0;
        if p > stop {
            break;
        }
        let size = lerp(from, to, p);
        set_size(&e, size);
        scene.advance(1);
        frames += 1;
        // Every box outside E rests where it rested.
        for (name, at) in &rest {
            if name == "E" || screen.inside.contains(&name.as_str()) {
                continue;
            }
            assert_eq!(&scene.bounds(name), at, "{name} moved at p {p}");
        }
        // E keeps its resting place and is laid out at the flight size.
        assert_eq!(
            scene.bounds("E"),
            [e_rest[0], e_rest[1], size.0, size.1],
            "E at p {p}"
        );
        // Its resting box, as the placeholder solved it, is its box at rest.
        assert_eq!(rect(resting_box(&e).unwrap()), e_rest, "rest at p {p}");
        // Its inside equals a fresh load with E authored at the flight size.
        let oracle = (screen.make)(size);
        oracle.advance(2);
        for name in screen.inside {
            assert_eq!(
                scene.bounds(name),
                oracle.bounds(name),
                "{name} at p {p}, size {size:?}"
            );
        }
        // One placeholder in the tree, with its baseline marker, nothing more.
        let (_, reachable, held, markers) = counts(&scene);
        assert_eq!(held, 1, "one placeholder at p {p}");
        assert!(markers <= 1, "at most one baseline marker at p {p}");
        assert_eq!(reachable, rest_counts.1 + 1 + markers, "tree size at p {p}");
    }
    end(&e);
    scene.advance(1);
    assert_eq!(boxes(&scene, screen.names), rest, "after the step");
    let after_world: Vec<_> = screen.names.iter().map(|n| world(&scene, n)).collect();
    assert_eq!(after_world, rest_world, "drawn places after the step");
    assert_eq!(
        counts(&scene),
        rest_counts,
        "the resting tree after the step"
    );
    assert_eq!(rect(resting_box(&e).unwrap()), e_rest);
    assert_eq!(dirty(&scene, screen.names), Vec::<String>::new());
    // A fresh load of the same Screen gives the same boxes.
    let fresh = (screen.make)(screen.own);
    fresh.advance(2);
    assert_eq!(boxes(&fresh, screen.names), rest, "fresh load");
    frames
}

#[test]
fn in_place_hold_scenes_rest_as_described() {
    let up = upper(UPPER_SIZE);
    up.advance(2);
    assert_eq!(up.bounds("N1"), [0.0, 0.0, 80.0, 100.0]);
    assert_eq!(up.bounds("E"), [90.0, 0.0, 120.0, 100.0]);
    assert_eq!(up.bounds("N2"), [220.0, 0.0, 80.0, 100.0]);
    let low = lower(LOWER_SIZE);
    low.advance(2);
    assert_eq!(low.bounds("E"), [120.0, 0.0, 160.0, 120.0]);
    // Scrolled 200 along: E's left edge draws 80 left of the viewport.
    let (x, _) = world(&low, "E");
    assert_eq!(x, -80.0, "E is half out of view");
    // The label wraps inside E: taller than one 19-point line.
    assert!(up.bounds("Label")[3] > 19.0, "{:?}", up.bounds("Label"));
}

#[test]
fn in_place_hold_forward_upper_copy_grows_from_the_lower_size() {
    assert_eq!(fly(&upper_screen(), LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

#[test]
fn in_place_hold_forward_lower_copy_flies_to_the_upper_size() {
    assert_eq!(fly(&lower_screen(), LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

#[test]
fn in_place_hold_back_both_copies() {
    assert_eq!(fly(&upper_screen(), UPPER_SIZE, LOWER_SIZE, 1.0), 21);
    assert_eq!(fly(&lower_screen(), UPPER_SIZE, LOWER_SIZE, 1.0), 21);
}

#[test]
fn in_place_hold_cancelled_halfway_both_copies() {
    assert_eq!(fly(&upper_screen(), LOWER_SIZE, UPPER_SIZE, 0.5), 11);
    assert_eq!(fly(&lower_screen(), LOWER_SIZE, UPPER_SIZE, 0.5), 11);
}

/// A size that makes the label wrap differently from both ends: the
/// inside follows, the neighbors don't.
#[test]
fn in_place_hold_wraps_the_text_at_the_flight_width() {
    checked(|| {
        let scene = upper(UPPER_SIZE);
        scene.advance(2);
        let e = scene.named("E");
        let lines = scene.bounds("Label")[3];
        assert!(begin(&e, (260.0, 100.0)));
        scene.advance(1);
        let wide = scene.bounds("Label")[3];
        assert!(wide < lines, "wider E, fewer lines: {wide} vs {lines}");
        assert_eq!(scene.bounds("N2"), [220.0, 0.0, 80.0, 100.0]);
        end(&e);
        scene.advance(1);
        assert_eq!(scene.bounds("Label")[3], lines);
    });
}

/// A slot step lays the element out as a unit: its own root at a definite
/// size. The held element's inside, relative to the element, equals that
/// unit's at every size the step passes through. An absolute child ("Pin")
/// resolves against the held element, as against a unit root.
#[test]
fn in_place_hold_inside_matches_a_unit_root() {
    let pinned = |size: (f32, f32)| {
        let mut children = inside();
        children.push(css(
            "Pin",
            [
                U(POSITION, 2),
                U(INSET_RIGHT_KIND, 1),
                F(INSET_RIGHT, 6.0),
                U(INSET_BOTTOM_KIND, 1),
                F(INSET_BOTTOM, 4.0),
                U(WIDTH_KIND, 1),
                F(WIDTH, 20.0),
                U(HEIGHT_KIND, 1),
                F(HEIGHT, 10.0),
            ],
            vec![],
        ));
        element_with(size, &[], children)
    };
    let upper_pinned = |size| upper_with(pinned(size));
    checked(|| {
        let scene = upper_pinned(UPPER_SIZE);
        scene.advance(2);
        let e = scene.named("E");
        assert!(begin(&e, LOWER_SIZE));
        for step in 0..=10 {
            let size = lerp(LOWER_SIZE, UPPER_SIZE, step as f32 / 10.0);
            set_size(&e, size);
            scene.advance(1);
            let unit = super::scene(size, pinned(size));
            unit.advance(2);
            assert_eq!(unit.bounds("E")[2..], [size.0, size.1]);
            for name in ["Fill", "Label", "Pin"] {
                assert_eq!(scene.bounds(name), unit.bounds(name), "{name} at {size:?}");
            }
            // Pin sits in E's bottom right corner.
            assert_eq!(
                scene.bounds("Pin"),
                [size.0 - 26.0, size.1 - 14.0, 20.0, 10.0],
                "Pin at {size:?}"
            );
        }
        end(&e);
        scene.advance(1);
    });
}

// ---------------------------------------------------------------------------
// Shapes the first test Screens leave out.

/// E is the third cell of a three-column grid, placed there by a
/// GridItemPlacement; N1 and N2 are placed automatically before it. The
/// placement moves to the placeholder.
#[test]
fn in_place_hold_a_grid_item() {
    const POINTS: u64 = 1;
    let make = |size: (f32, f32)| {
        scene(
            (400.0, 300.0),
            B {
                tracks: vec![(0, POINTS, 80.0), (0, POINTS, 100.0), (0, POINTS, 200.0)],
                ..css(
                    "Grid",
                    [
                        [U(DISPLAY, 2), F(COLUMN_GAP, 10.0)].as_slice(),
                        &width(400.0),
                        &height(300.0),
                    ]
                    .concat(),
                    vec![
                        B {
                            placement: Some((3, 1)),
                            ..element(size)
                        },
                        sized("N1", 80.0, 100.0),
                        sized("N2", 80.0, 100.0),
                    ],
                )
            },
        )
    };
    let names = ["Grid", "N1", "N2", "E", "Fill", "Label"];
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert_eq!(rest.bounds("N1")[0], 0.0);
    assert_eq!(rest.bounds("N2")[0], 90.0);
    assert_eq!(rest.bounds("E")[..2], [200.0, 0.0], "E in the third column");
    let screen = Screen {
        make: &make,
        names: &names,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

/// E is absolute in a positioned grid, placed in its second column: its
/// containing block is that grid area, 300 wide, so its 10% top padding is
/// 30, at rest and held.
#[test]
fn in_place_hold_an_absolute_grid_item_pads_against_its_area() {
    const POINTS: u64 = 1;
    const PADDING_TOP_PERCENT: u16 = 61064;
    let make = |size: (f32, f32)| {
        scene(
            (400.0, 300.0),
            B {
                tracks: vec![(0, POINTS, 100.0), (0, POINTS, 300.0)],
                ..css(
                    "Grid",
                    [
                        [U(DISPLAY, 2), U(POSITION, 1)].as_slice(),
                        &width(400.0),
                        &height(300.0),
                    ]
                    .concat(),
                    vec![
                        sized("N1", 80.0, 100.0),
                        B {
                            placement: Some((2, 1)),
                            ..css(
                                "E",
                                [
                                    [
                                        U(DISPLAY, 1),
                                        U(FLEX_DIRECTION, 2),
                                        U(BOX_SIZING, 1),
                                        U(POSITION, 2),
                                        F(PADDING_TOP_PERCENT, 10.0),
                                    ]
                                    .as_slice(),
                                    &width(size.0),
                                    &height(size.1),
                                ]
                                .concat(),
                                inside(),
                            )
                        },
                    ],
                )
            },
        )
    };
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert_eq!(rest.bounds("E")[0], 100.0, "E in the second column");
    assert_eq!(rest.bounds("Fill")[1], 30.0, "10% of the area's 300");
    let screen = Screen {
        make: &make,
        names: &["Grid", "N1", "E", "Fill", "Label"],
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    // The same size first: nothing moves at all.
    assert_eq!(fly(&screen, UPPER_SIZE, UPPER_SIZE, 1.0), 21);
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

/// E has margins and is relatively positioned with insets: the placeholder
/// keeps both, and E publishes the placeholder's place.
#[test]
fn in_place_hold_margins_and_relative_insets() {
    let extra = [
        F(MARGIN_LEFT, 15.0),
        F(MARGIN_TOP, 5.0),
        U(POSITION, 1),
        U(INSET_LEFT_KIND, 1),
        F(INSET_LEFT, 7.0),
        U(INSET_TOP_KIND, 1),
        F(INSET_TOP, 4.0),
    ];
    let make = move |size| upper_with(element_with(size, &extra, inside()));
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert_eq!(rest.bounds("E")[..2], [80.0 + 10.0 + 15.0 + 7.0, 5.0 + 4.0]);
    assert_eq!(rest.bounds("N2")[0], 80.0 + 10.0 + 15.0 + 120.0 + 10.0);
    let screen = Screen {
        make: &make,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
    assert_eq!(fly(&screen, UPPER_SIZE, LOWER_SIZE, 0.5), 11);
}

/// E's top and left padding are percentages, which resolve against the
/// row's width: held, E's containing block is its placeholder, yet its
/// inside sits as a fresh load at the flight size lays it out.
#[test]
fn in_place_hold_percentage_padding() {
    const PADDING_TOP_PERCENT: u16 = 61064;
    const PADDING_LEFT_PERCENT: u16 = 61070;
    let make = |size: (f32, f32)| {
        upper_with(css(
            "E",
            [
                [
                    U(DISPLAY, 1),
                    U(FLEX_DIRECTION, 2),
                    U(BOX_SIZING, 1),
                    F(FLEX_SHRINK, 0.0),
                    F(PADDING_RIGHT, 10.0),
                    F(PADDING_BOTTOM, 10.0),
                    F(PADDING_LEFT_PERCENT, 5.0),
                    F(PADDING_TOP_PERCENT, 5.0),
                ]
                .as_slice(),
                &width(size.0),
                &height(size.1),
            ]
            .concat(),
            inside(),
        ))
    };
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    // Fill, in E, sits inside 5% of the row's 400.
    assert_eq!(
        rest.bounds("Fill")[..2],
        [20.0, 20.0],
        "the percentage resolves against the row"
    );
    let screen = Screen {
        make: &make,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
    assert_eq!(fly(&screen, UPPER_SIZE, LOWER_SIZE, 0.5), 11);
}

/// E's top padding is 0% at take-off and keyed to 5% halfway through the
/// flight: it resolves against the row's width, as at rest, not against
/// the placeholder.
#[test]
fn in_place_hold_a_percentage_keyed_from_zero() {
    const PADDING_TOP_PERCENT: u16 = 61064;
    let make = |size: (f32, f32), top: f32| {
        upper_with(css(
            "E",
            [
                [
                    U(DISPLAY, 1),
                    U(FLEX_DIRECTION, 2),
                    U(BOX_SIZING, 1),
                    F(FLEX_SHRINK, 0.0),
                    F(PADDING_RIGHT, 10.0),
                    F(PADDING_BOTTOM, 10.0),
                    F(PADDING_LEFT, 10.0),
                    F(PADDING_TOP_PERCENT, top),
                ]
                .as_slice(),
                &width(size.0),
                &height(size.1),
            ]
            .concat(),
            inside(),
        ))
    };
    checked(|| {
        let scene = make(UPPER_SIZE, 0.0);
        scene.advance(2);
        let e_rest = scene.bounds("E");
        let n2 = scene.bounds("N2");
        let e = scene.named("E");
        assert!(begin(&e, LOWER_SIZE));
        for step in 0..=10 {
            let p = step as f32 / 10.0;
            let top = if step < 5 { 0.0 } else { 5.0 };
            if step == 5 {
                assert!(CoreRegistry::set_double_handle(
                    &scene.record("E"),
                    PADDING_TOP_PERCENT.into(),
                    5.0
                ));
            }
            let size = lerp(LOWER_SIZE, UPPER_SIZE, p);
            set_size(&e, size);
            scene.advance(1);
            let oracle = make(size, top);
            oracle.advance(2);
            for name in INSIDE {
                assert_eq!(scene.bounds(name), oracle.bounds(name), "{name} at p {p}");
            }
            assert_eq!(scene.bounds("N2"), n2, "N2 at p {p}");
            assert_eq!(scene.bounds("E")[..2], e_rest[..2], "E at p {p}");
        }
        // 5% of the row's 400.
        assert_eq!(scene.bounds("Fill")[1], 20.0);
        end(&e);
        scene.advance(1);
    });
}

/// E flies far wider than it rests, inside the lower Screen's scroll view:
/// the flight reaches no ancestor's scroll range.
#[test]
fn in_place_hold_a_larger_flight_leaves_scroll_ranges_alone() {
    checked(|| {
        let scene = lower(LOWER_SIZE);
        scene.advance(2);
        let range = |scene: &Scene| {
            scene
                .named("Viewport")
                .with_downcast::<LayoutComponent, _>(|layout| layout.scroll_range())
                .unwrap()
        };
        let rest = range(&scene);
        assert_eq!((rest.x, rest.y), (200.0, 0.0), "the strip scrolls");
        let e = scene.named("E");
        assert!(begin(&e, LOWER_SIZE));
        for size in [(600.0, 120.0), (600.0, 400.0)] {
            set_size(&e, size);
            scene.advance(1);
            assert_eq!(scene.bounds("E")[2..], [size.0, size.1]);
            let held = range(&scene);
            assert_eq!((held.x, held.y), (rest.x, rest.y), "at {size:?}");
        }
        end(&e);
        scene.advance(1);
        let after = range(&scene);
        assert_eq!((after.x, after.y), (rest.x, rest.y));
    });
}

/// E is content-box. The flight size is its CSS width and height under its
/// own box-sizing, as the slot path sizes a unit's root, so its border box
/// is the flight size plus its padding, and padding keyed during the flight
/// shows the next frame. Its inside equals a fresh load of E authored at
/// the flight size with that padding; its neighbors stay put.
#[test]
fn in_place_hold_sizes_a_content_box_element_as_the_slot_path_does() {
    let make = |size: (f32, f32), top: f32| {
        upper_with(css(
            "E",
            [
                [
                    U(DISPLAY, 1),
                    U(FLEX_DIRECTION, 2),
                    U(BOX_SIZING, 0),
                    F(FLEX_SHRINK, 0.0),
                    F(PADDING_RIGHT, 10.0),
                    F(PADDING_BOTTOM, 10.0),
                    F(PADDING_LEFT, 10.0),
                    F(PADDING_TOP, top),
                ]
                .as_slice(),
                &width(size.0),
                &height(size.1),
            ]
            .concat(),
            inside(),
        ))
    };
    checked(|| {
        let (from, to) = ((100.0, 80.0), (140.0, 100.0));
        let scene = make(from, 10.0);
        scene.advance(2);
        let e_rest = scene.bounds("E");
        assert_eq!(e_rest[2..], [120.0, 100.0], "a content box");
        let n2 = scene.bounds("N2");
        let e = scene.named("E");
        assert!(begin(&e, from));
        for step in 0..=10 {
            let p = step as f32 / 10.0;
            // Padding keyed during the flight, from halfway.
            let top = if step < 5 { 10.0 } else { 13.0 };
            if step == 5 {
                assert!(CoreRegistry::set_double_handle(
                    &scene.record("E"),
                    PADDING_TOP.into(),
                    13.0
                ));
            }
            let size = lerp(from, to, p);
            set_size(&e, size);
            scene.advance(1);
            assert_eq!(
                scene.bounds("E"),
                [e_rest[0], e_rest[1], size.0 + 20.0, size.1 + 10.0 + top],
                "E at p {p}"
            );
            let oracle = make(size, top);
            oracle.advance(2);
            for name in INSIDE {
                assert_eq!(scene.bounds(name), oracle.bounds(name), "{name} at p {p}");
            }
            assert_eq!(scene.bounds("N2"), n2, "N2 at p {p}");
            assert_eq!(rect(resting_box(&e).unwrap()), e_rest, "rest at p {p}");
        }
        end(&e);
        scene.advance(1);
        // At rest again with the keyed padding: a fresh load's boxes.
        let fresh = make(from, 13.0);
        fresh.advance(2);
        assert_eq!(boxes(&scene, &UPPER_NAMES), boxes(&fresh, &UPPER_NAMES));
    });
}

/// The contract between the file and the hold (pr-a-brief.md section 9,
/// finding 1, and the coordinator's ruling): a matched element is the
/// containing block of its absolutely placed descendants, at rest and in
/// flight. The file makes a keyed element positioned at rest; given that,
/// no absolute child moves at take-off or landing. A file that leaves the
/// element static breaks the contract: its absolute child is placed against
/// a farther ancestor at rest and against the element while held, so it
/// moves at take-off and back at landing.
#[test]
fn in_place_a_static_element_with_an_absolute_child_is_placed_as_documented() {
    let pinned = |position: u64| {
        let mut children = inside();
        children.push(css(
            "Pin",
            [
                U(POSITION, 2),
                U(INSET_RIGHT_KIND, 1),
                F(INSET_RIGHT, 6.0),
                U(INSET_BOTTOM_KIND, 1),
                F(INSET_BOTTOM, 4.0),
                U(WIDTH_KIND, 1),
                F(WIDTH, 20.0),
                U(HEIGHT_KIND, 1),
                F(HEIGHT, 10.0),
            ],
            vec![],
        ));
        upper_with(element_with(UPPER_SIZE, &[U(POSITION, position)], children))
    };
    // Take-off and landing at the resting size: only the hold changes.
    let around = |position: u64| {
        checked(|| {
            let scene = pinned(position);
            scene.advance(2);
            let rest = world(&scene, "Pin");
            let e = scene.named("E");
            assert!(begin(&e, UPPER_SIZE));
            scene.advance(1);
            let held = world(&scene, "Pin");
            end(&e);
            scene.advance(1);
            assert_eq!(world(&scene, "Pin"), rest, "landing puts Pin back");
            (rest, held)
        })
    };
    // Kept: E relative, as the file makes a keyed element.
    let (rest, held) = around(1);
    assert_eq!(held, rest, "nothing moves at take-off");
    let e_at = |position| {
        let scene = pinned(position);
        scene.advance(2);
        world(&scene, "E")
    };
    let e = e_at(1);
    assert_eq!(
        rest,
        (e.0 + UPPER_SIZE.0 - 26.0, e.1 + UPPER_SIZE.1 - 14.0),
        "Pin in E's corner"
    );
    // Broken: E static, so at rest Pin is placed against the Screen.
    let (rest, held) = around(0);
    assert_eq!(
        rest,
        (400.0 - 26.0, 300.0 - 14.0),
        "Pin in the Screen's corner"
    );
    let e = e_at(0);
    assert_eq!(
        held,
        (e.0 + UPPER_SIZE.0 - 26.0, e.1 + UPPER_SIZE.1 - 14.0),
        "held, Pin moves to E's corner"
    );
}

/// The row aligns its items by their baselines. E, a flex column, has a
/// baseline of its own, which places it below its neighbors' synthesized
/// ones; held, its placeholder reports the baseline E rested with, so
/// nothing in the row moves.
#[test]
fn in_place_hold_in_a_baseline_aligned_row() {
    const BASELINE: u64 = 9;
    let make = |size| {
        scene(
            (400.0, 300.0),
            css(
                "Frame",
                [
                    [U(DISPLAY, 1), U(ALIGN_ITEMS, BASELINE), F(COLUMN_GAP, 10.0)].as_slice(),
                    &width(400.0),
                    &height(300.0),
                ]
                .concat(),
                vec![
                    sized("N1", 80.0, 100.0),
                    element(size),
                    sized("N2", 80.0, 100.0),
                ],
            ),
        )
    };
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert!(
        rest.bounds("E")[1] > 0.0,
        "E's baseline places it: {:?}",
        rest.bounds("E")
    );
    let screen = Screen {
        make: &make,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
    assert_eq!(fly(&screen, UPPER_SIZE, LOWER_SIZE, 0.5), 11);
    // The placeholder holds a baseline marker.
    let scene = make(UPPER_SIZE);
    scene.advance(2);
    let e = scene.named("E");
    assert!(begin(&e, LOWER_SIZE));
    scene.advance(1);
    assert_eq!(counts(&scene).3, 1);
    end(&e);
    scene.advance(1);
    assert_eq!(counts(&scene).3, 0);
}

/// A baseline-aligned row whose element is a block with percentage top
/// padding: its baseline sits under that padding, which resolves against
/// the row, so the placeholder reports the baseline at the same depth.
#[test]
fn in_place_hold_in_a_baseline_aligned_row_with_percentage_padding() {
    const BASELINE: u64 = 9;
    const PADDING_TOP_PERCENT: u16 = 61064;
    let make = |size: (f32, f32)| {
        scene(
            (400.0, 300.0),
            css(
                "Frame",
                [
                    [U(DISPLAY, 1), U(ALIGN_ITEMS, BASELINE), F(COLUMN_GAP, 10.0)].as_slice(),
                    &width(400.0),
                    &height(300.0),
                ]
                .concat(),
                vec![
                    sized("N1", 80.0, 100.0),
                    css(
                        "E",
                        [
                            [
                                U(BOX_SIZING, 1),
                                F(FLEX_SHRINK, 0.0),
                                F(PADDING_TOP_PERCENT, 5.0),
                            ]
                            .as_slice(),
                            &width(size.0),
                            &height(size.1),
                        ]
                        .concat(),
                        vec![css(
                            "Inner",
                            [U(DISPLAY, 1)],
                            vec![sized("Dot", 20.0, 20.0)],
                        )],
                    ),
                    sized("N2", 80.0, 100.0),
                ],
            ),
        )
    };
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    // E's baseline is Inner's (20) under its top padding (5% of 400), the
    // neighbors' their bottoms (100).
    assert_eq!(rest.bounds("E")[1], 60.0, "E's baseline places it");
    let screen = Screen {
        make: &make,
        names: &["Frame", "N1", "E", "N2", "Inner", "Dot"],
        inside: &["Inner", "Dot"],
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
    assert_eq!(fly(&screen, UPPER_SIZE, LOWER_SIZE, 0.5), 11);
}

/// E sits in a plain group inside the row: layout sees through the group,
/// as through CSS's `display: contents` (which the compiler writes as no
/// box at all).
#[test]
fn in_place_hold_inside_a_group_layout_sees_through() {
    let make = |size| {
        upper_with(B {
            in_group: true,
            ..element(size)
        })
    };
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert_eq!(rest.bounds("E"), [90.0, 0.0, 120.0, 100.0]);
    let screen = Screen {
        make: &make,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

/// E is absolute: already out of flow. The placeholder is absolute too and
/// holds its insets; N1 and N2 close up as they do at rest.
#[test]
fn in_place_hold_an_absolute_element() {
    let extra = [
        U(POSITION, 2),
        U(INSET_LEFT_KIND, 1),
        F(INSET_LEFT, 150.0),
        U(INSET_TOP_KIND, 1),
        F(INSET_TOP, 120.0),
    ];
    let make = move |size| upper_with(element_with(size, &extra, inside()));
    let rest = make(UPPER_SIZE);
    rest.advance(2);
    assert_eq!(rest.bounds("E")[..2], [150.0, 120.0]);
    assert_eq!(rest.bounds("N2")[0], 90.0, "E is out of the row's flow");
    let screen = Screen {
        make: &make,
        names: &UPPER_NAMES,
        inside: &INSIDE,
        own: UPPER_SIZE,
    };
    assert_eq!(fly(&screen, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

/// E's own layout animation is running when the hold begins: while held,
/// and in the frame the hold ends, E's own box takes its layout at once.
#[test]
fn in_place_hold_over_a_running_layout_animation() {
    checked(|| {
        let scene = upper(UPPER_SIZE);
        scene.advance(2);
        let e = scene.named("E");
        super::enable_leaf_layout_interpolation(&e);
        // Start E's own animation: a new authored width.
        let record = scene.record("E");
        assert!(CoreRegistry::set_double_handle(
            &record,
            WIDTH.into(),
            140.0
        ));
        // Interpolation reads its clock before advancing it.
        scene.advance(3);
        let moving = scene.bounds("E");
        assert!(
            moving[2] > 120.0 && moving[2] < 140.0,
            "E animates: {moving:?}"
        );
        // The hold rests E where layout put it: the animation's end.
        let rest = rect(resting_box(&e).unwrap());
        assert_eq!(rest[2], 140.0);
        assert!(begin(&e, LOWER_SIZE));
        let n2 = scene.bounds("N2");
        for step in 0..=10 {
            let size = lerp(LOWER_SIZE, UPPER_SIZE, step as f32 / 10.0);
            set_size(&e, size);
            scene.advance(1);
            assert_eq!(
                scene.bounds("E"),
                [rest[0], rest[1], size.0, size.1],
                "at {size:?}"
            );
            assert_eq!(rect(resting_box(&e).unwrap()), rest);
            assert_eq!(scene.bounds("N2"), n2);
            assert_eq!(counts(&scene).2, 1);
        }
        end(&e);
        scene.advance(1);
        assert_eq!(scene.bounds("E"), rest, "E takes its rest at once");
        scene.advance(60);
        assert_eq!(scene.bounds("E"), rest);
        assert_eq!(counts(&scene).2, 0);
    });
}

/// A child inside the held element is hidden and shown again during the
/// step: the inside re-solves under the hold, as a fresh load would lay it
/// out, and the neighbors stay put.
#[test]
fn in_place_hold_a_structural_change_inside() {
    let make = |size: (f32, f32), fill_hidden: bool| {
        let mut children = inside();
        if fill_hidden {
            children[0].css.as_mut().unwrap().push(U(DISPLAY, 3));
        }
        upper_with(element_with(size, &[], children))
    };
    checked(|| {
        let scene = make(UPPER_SIZE, false);
        scene.advance(2);
        let rest = boxes(&scene, &UPPER_NAMES);
        let rest_counts = counts(&scene);
        let e = scene.named("E");
        let fill = scene.record("Fill");
        assert!(begin(&e, LOWER_SIZE));
        for (step, hidden) in [(0, false), (1, true), (2, true), (3, false), (4, false)] {
            let size = lerp(LOWER_SIZE, UPPER_SIZE, step as f32 / 4.0);
            set_size(&e, size);
            assert!(CoreRegistry::set_uint_handle(
                &fill,
                DISPLAY.into(),
                if hidden { 3 } else { 0 }
            ));
            scene.advance(1);
            for name in ["Frame", "N1", "N2"] {
                assert_eq!(
                    scene.bounds(name),
                    rest.iter().find(|r| r.0 == name).unwrap().1
                );
            }
            let oracle = make(size, hidden);
            oracle.advance(2);
            assert_eq!(
                scene.bounds("Label"),
                oracle.bounds("Label"),
                "Label at step {step}"
            );
            if !hidden {
                assert_eq!(
                    scene.bounds("Fill"),
                    oracle.bounds("Fill"),
                    "Fill at step {step}"
                );
            }
            assert_eq!(counts(&scene).2, 1);
        }
        end(&e);
        scene.advance(1);
        assert_eq!(boxes(&scene, &UPPER_NAMES), rest);
        assert_eq!(counts(&scene), rest_counts);
    });
}

/// The rest is frozen at take-off: E is 30% of a row that grows during the
/// step. E's rest keeps its take-off width while the row's other boxes
/// follow the new row; when the hold ends E takes its new width.
#[test]
fn in_place_hold_keeps_its_rest_through_a_resize() {
    // E authored at 30% of the frame, or at a length.
    let make = |frame: f32, e: Option<(f32, f32)>| {
        let e = match e {
            Some(size) => element(size),
            None => css(
                "E",
                [
                    element_props().as_slice(),
                    &[U(WIDTH_KIND, 1), F(WIDTH, 0.0), F(WIDTH_PERCENT, 30.0)],
                    &height(100.0),
                ]
                .concat(),
                inside(),
            ),
        };
        scene(
            (800.0, 300.0),
            css(
                "Frame",
                [
                    [U(DISPLAY, 1), U(ALIGN_ITEMS, 2), F(COLUMN_GAP, 10.0)].as_slice(),
                    &width(frame),
                    &height(300.0),
                ]
                .concat(),
                vec![sized("N1", 80.0, 100.0), e, sized("N2", 80.0, 100.0)],
            ),
        )
    };
    checked(|| {
        let scene = make(400.0, None);
        scene.advance(2);
        let e_rest = scene.bounds("E");
        assert_eq!(e_rest[..2], [90.0, 0.0]);
        assert!((e_rest[2] - 120.0).abs() < 1e-3, "30% of 400: {e_rest:?}");
        let e = scene.named("E");
        // Where layout solved E, which the hold freezes.
        let solved = rect(resting_box(&e).unwrap());
        assert!(begin(&e, LOWER_SIZE));
        scene.advance(1);
        assert!(CoreRegistry::set_double_handle(
            &scene.record("Frame"),
            WIDTH.into(),
            600.0
        ));
        for _ in 0..3 {
            scene.advance(1);
            // The row is 600 wide; E keeps resting at its take-off width.
            assert_eq!(scene.bounds("Frame")[2], 600.0);
            assert_eq!(rect(resting_box(&e).unwrap()), solved);
            let frozen = make(600.0, Some((solved[2], solved[3])));
            frozen.advance(2);
            for name in ["Frame", "N1", "N2"] {
                assert_eq!(scene.bounds(name), frozen.bounds(name), "{name}");
            }
            assert_eq!(scene.bounds("E"), [90.0, 0.0, LOWER_SIZE.0, LOWER_SIZE.1]);
        }
        end(&e);
        scene.advance(1);
        // At rest E is 30% of 600.
        let fresh = make(600.0, None);
        fresh.advance(2);
        assert_eq!(fresh.bounds("E")[2], 180.0);
        assert_eq!(boxes(&scene, &UPPER_NAMES), boxes(&fresh, &UPPER_NAMES));
    });
}

// ---------------------------------------------------------------------------
// A participant node: an image.

fn participant_bounds(handle: &CoreHandle) -> [f32; 4] {
    let bounds = handle
        .with_downcast::<LayoutParticipant, _>(|participant| participant.layout_bounds())
        .expect("a participant");
    [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
}

/// The image's participant, "Picture"'s.
fn picture(scene: &Scene) -> CoreHandle {
    let named = |host: CoreHandle| {
        host.with(|object| {
            object
                .as_component()
                .is_some_and(|component| component.base.name() == "Picture")
        })
        .unwrap_or(false)
    };
    scene
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<LayoutParticipant>())
        .into_iter()
        .find(|handle| {
            handle
                .with_downcast::<LayoutParticipant, _>(LayoutParticipant::measurement_host_handle)
                .flatten()
                .is_some_and(named)
        })
        .expect("the picture's participant")
}

/// The image "Picture" starts the row, before N1, at its natural 100 by 30
/// size, and flies in place as a box does.
#[test]
fn in_place_hold_an_image() {
    checked(|| {
        let scene = scene(
            (400.0, 300.0),
            B {
                image_participants: vec![("Picture", vec![F(FLEX_SHRINK, 0.0)])],
                ..css(
                    "Frame",
                    [
                        [U(DISPLAY, 1), U(ALIGN_ITEMS, 2), F(COLUMN_GAP, 10.0)].as_slice(),
                        &width(400.0),
                        &height(300.0),
                    ]
                    .concat(),
                    vec![sized("N1", 80.0, 100.0)],
                )
            },
        );
        scene.advance(2);
        let image = picture(&scene);
        let rest = participant_bounds(&image);
        assert_eq!(
            rest,
            [0.0, 0.0, 100.0, 30.0],
            "the image at its natural size"
        );
        let n1 = scene.bounds("N1");
        assert_eq!(n1, [110.0, 0.0, 80.0, 100.0]);
        let rest_counts = counts(&scene);
        assert!(begin(&image, (100.0, 30.0)));
        for step in 0..=10 {
            let size = lerp((100.0, 30.0), (200.0, 60.0), step as f32 / 10.0);
            set_size(&image, size);
            scene.advance(1);
            assert_eq!(
                participant_bounds(&image),
                [0.0, 0.0, size.0, size.1],
                "at {size:?}"
            );
            assert_eq!(rect(resting_box(&image).unwrap()), rest);
            assert_eq!(scene.bounds("N1"), n1);
            assert_eq!(scene.bounds("Frame"), [0.0, 0.0, 400.0, 300.0]);
            assert_eq!(counts(&scene).2, 1);
        }
        end(&image);
        scene.advance(1);
        assert_eq!(participant_bounds(&image), rest);
        assert_eq!(counts(&scene), rest_counts);
    });
}

/// An image caught mid-retarget by its own interpolation: the hold snaps it
/// past both animations, it flies at its resting place, and when the hold
/// ends nothing it snapped past resumes.
#[test]
fn in_place_hold_an_image_over_a_retargeted_animation() {
    checked(|| {
        let scene = scene(
            (400.0, 300.0),
            B {
                image_participants: vec![("Picture", vec![F(FLEX_SHRINK, 0.0)])],
                ..css(
                    "Frame",
                    [
                        [U(DISPLAY, 1), U(ALIGN_ITEMS, 2), F(COLUMN_GAP, 10.0)].as_slice(),
                        &width(400.0),
                        &height(300.0),
                    ]
                    .concat(),
                    vec![sized("N1", 80.0, 100.0)],
                )
            },
        );
        scene.advance(2);
        let frame = scene.named("Frame");
        super::enable_leaf_layout_interpolation(&frame);
        let image = picture(&scene);
        image.with_downcast_mut::<LayoutParticipant, _>(|participant| {
            use crate::source::layout::layout_enums::{LayoutDirection, LayoutStyleInterpolation};
            participant.cascade_layout_style(
                LayoutStyleInterpolation::Linear,
                None,
                1.0,
                LayoutDirection::Ltr,
            );
        });
        let record = scene.record("Frame");
        let pad = |value: f32| {
            assert!(CoreRegistry::set_double_handle(
                &record,
                PADDING_LEFT.into(),
                value
            ));
        };
        // The image animates toward 40, then is retargeted to 80 mid-way.
        pad(40.0);
        scene.advance(3);
        let moving = participant_bounds(&image);
        assert!(
            moving[0] > 0.0 && moving[0] < 40.0,
            "the image animates: {moving:?}"
        );
        pad(80.0);
        scene.advance(3);
        let moving = participant_bounds(&image);
        assert!(
            moving[0] > 0.0 && moving[0] < 80.0,
            "the image smooths toward its new target: {moving:?}"
        );
        let rest = rect(resting_box(&image).unwrap());
        assert_eq!(rest, [80.0, 0.0, 100.0, 30.0], "layout's rest: the end");
        assert!(begin(&image, (100.0, 30.0)));
        for step in 0..=10 {
            let size = lerp((100.0, 30.0), (200.0, 60.0), step as f32 / 10.0);
            set_size(&image, size);
            scene.advance(1);
            assert_eq!(
                participant_bounds(&image),
                [rest[0], rest[1], size.0, size.1],
                "at {size:?}"
            );
        }
        end(&image);
        scene.advance(1);
        assert_eq!(participant_bounds(&image), rest, "the image rests at once");
        for _ in 0..60 {
            scene.advance(1);
            assert_eq!(participant_bounds(&image), rest, "nothing resumes");
        }
    });
}

/// The resting box of an element that is not held is its own solved box;
/// a Screen's root has no layout parent and is never held.
#[test]
fn in_place_hold_resting_box_and_refusals() {
    use crate::css_layout::flight_hold::holds_begun;
    let scene = upper(UPPER_SIZE);
    scene.advance(2);
    let e = scene.named("E");
    assert_eq!(rect(resting_box(&e).unwrap()), scene.bounds("E"));
    let holds = holds_begun();
    assert!(
        !begin(&scene.artboard.core_handle(), UPPER_SIZE),
        "the root"
    );
    assert_eq!(holds_begun(), holds);
    assert_eq!(counts(&scene).2, 0);
}
