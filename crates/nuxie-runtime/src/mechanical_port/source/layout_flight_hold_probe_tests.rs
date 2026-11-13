//! PROBE (one shape for a Screen): the layout hold, css_layout/flight_hold.rs.
//!
//! The matched element "E" stays where it was authored. On the upper Screen
//! it sits in a flex row between N1 and N2; on the lower Screen it sits in a
//! horizontal scroll view, scrolled so it is half out of view, between M1
//! and M2. E holds a child that stretches ("Fill", flex-grow 1 in a column)
//! and a text that wraps ("Label", 16 point words about 197 points wide).
//!
//! Oracle: the same Screen loaded fresh with E authored at the flight size.
//! E's inside must be laid out exactly as it is there, and every other box
//! exactly as it rests, on every frame of a step, forward, Back and when
//! cancelled halfway.
use super::*;

const SCROLL_OFFSET_X: u16 = 759;
const ALIGN_ITEMS: u16 = 61013;
const FLEX_DIRECTION: u16 = 61006;
const MIN_HEIGHT_KIND: u16 = 61030;

/// E: a border-box flex column at `size` with 10 points of padding, holding
/// Fill and Label.
fn element(size: (f32, f32)) -> B {
    css(
        "E",
        [
            [
                U(DISPLAY, 1),
                U(FLEX_DIRECTION, 2),
                U(BOX_SIZING, 1),
                F(FLEX_SHRINK, 0.0),
            ]
            .as_slice(),
            &width(size.0),
            &height(size.1),
            &padding(10.0),
        ]
        .concat(),
        vec![
            css(
                "Fill",
                [F(FLEX_GROW, 1.0), U(MIN_HEIGHT_KIND, 1)],
                vec![],
            ),
            B {
                link: Some("tap"),
                ..css("Label", [], vec![])
            },
        ],
    )
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
            vec![sized("N1", 80.0, 100.0), element(e), sized("N2", 80.0, 100.0)],
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
                        vec![sized("M1", 120.0, 100.0), element(e), sized("M2", 120.0, 100.0)],
                    )
                }],
            )],
        ),
    )
}

const UPPER_NAMES: [&str; 6] = ["Frame", "N1", "E", "N2", "Fill", "Label"];
const LOWER_NAMES: [&str; 8] = ["Frame", "Viewport", "Strip", "M1", "E", "M2", "Fill", "Label"];
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

fn handle(scene: &Scene, name: &str) -> CoreHandle {
    scene.named(name)
}

fn counts(scene: &Scene) -> (usize, usize, usize) {
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

/// The draw positions a box's own paint reaches: its world translation.
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

struct Screen {
    make: fn((f32, f32)) -> Scene,
    names: &'static [&'static str],
    own: (f32, f32),
}
const UPPER: Screen = Screen {
    make: upper,
    names: &UPPER_NAMES,
    own: UPPER_SIZE,
};
const LOWER: Screen = Screen {
    make: lower,
    names: &LOWER_NAMES,
    own: LOWER_SIZE,
};

/// One copy flown from `from` to `to` over `samples` frames, cut at `stop`
/// (1 lands; less cancels there). Checks every frame against the oracle and
/// returns the number of frames checked.
fn fly(screen: &Screen, from: (f32, f32), to: (f32, f32), stop: f32) -> usize {
    let scene = (screen.make)(screen.own);
    scene.advance(2);
    let rest = boxes(&scene, screen.names);
    let rest_world: Vec<_> = screen.names.iter().map(|n| world(&scene, n)).collect();
    let rest_counts = counts(&scene);
    let e = handle(&scene, "E");
    let e_rest = scene.bounds("E");
    assert!(LayoutComponent::begin_flight_hold(&e, from));
    let mut frames = 0;
    for step in 0..=20 {
        let p = step as f32 / 20.0;
        if p > stop {
            break;
        }
        let size = lerp(from, to, p);
        LayoutComponent::set_flight_hold_size(&e, size);
        scene.advance(1);
        frames += 1;
        // Every box but E and its inside rests where it rested.
        for (name, at) in &rest {
            if matches!(name.as_str(), "E" | "Fill" | "Label") {
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
        let held = LayoutComponent::flight_hold_rest(&e).expect("a held rest");
        assert_eq!(
            [held.left(), held.top(), held.width(), held.height()],
            e_rest,
            "rest at p {p}"
        );
        // Its inside equals a fresh load with E authored at the flight size.
        let oracle = (screen.make)(size);
        oracle.advance(2);
        for name in ["Fill", "Label"] {
            assert_eq!(scene.bounds(name), oracle.bounds(name), "{name} at p {p}, size {size:?}");
        }
        // One placeholder in the tree, nothing more.
        let (_, reachable, held) = counts(&scene);
        assert_eq!(held, 1, "one placeholder at p {p}");
        assert_eq!(reachable, rest_counts.1 + 1, "tree size at p {p}");
    }
    LayoutComponent::end_flight_hold(&e);
    scene.advance(1);
    assert_eq!(boxes(&scene, screen.names), rest, "after the step");
    let after_world: Vec<_> = screen.names.iter().map(|n| world(&scene, n)).collect();
    assert_eq!(after_world, rest_world, "drawn places after the step");
    assert_eq!(counts(&scene), rest_counts, "the resting tree after the step");
    assert!(LayoutComponent::flight_hold_rest(&e).is_none());
    assert_eq!(dirty(&scene, screen.names), Vec::<String>::new());
    // A fresh load of the same Screen gives the same boxes.
    let fresh = (screen.make)(screen.own);
    fresh.advance(2);
    assert_eq!(boxes(&fresh, screen.names), rest, "fresh load");
    frames
}

#[test]
fn probe_the_scenes_rest_as_described() {
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
fn probe_hold_forward_upper_copy_grows_from_the_lower_size() {
    assert_eq!(fly(&UPPER, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

#[test]
fn probe_hold_forward_lower_copy_flies_to_the_upper_size() {
    assert_eq!(fly(&LOWER, LOWER_SIZE, UPPER_SIZE, 1.0), 21);
}

#[test]
fn probe_hold_back_both_copies() {
    assert_eq!(fly(&UPPER, UPPER_SIZE, LOWER_SIZE, 1.0), 21);
    assert_eq!(fly(&LOWER, UPPER_SIZE, LOWER_SIZE, 1.0), 21);
}

#[test]
fn probe_hold_cancelled_halfway_both_copies() {
    assert_eq!(fly(&UPPER, LOWER_SIZE, UPPER_SIZE, 0.5), 11);
    assert_eq!(fly(&LOWER, LOWER_SIZE, UPPER_SIZE, 0.5), 11);
}

/// A size that makes the label wrap differently from both ends: the
/// inside follows, the neighbors don't.
#[test]
fn probe_hold_wraps_the_text_at_the_flight_width() {
    let scene = upper(UPPER_SIZE);
    scene.advance(2);
    let e = handle(&scene, "E");
    let one_line = scene.bounds("Label")[3];
    assert!(LayoutComponent::begin_flight_hold(&e, (260.0, 100.0)));
    scene.advance(1);
    let wide = scene.bounds("Label")[3];
    assert!(wide < one_line, "wider E, fewer lines: {wide} vs {one_line}");
    assert_eq!(scene.bounds("N2"), [220.0, 0.0, 80.0, 100.0]);
    LayoutComponent::end_flight_hold(&e);
    scene.advance(1);
    assert_eq!(scene.bounds("Label")[3], one_line);
}
