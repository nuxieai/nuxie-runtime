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
    upper_with(element(e), vec![])
}

fn upper_with(e: B, extra: Vec<B>) -> Scene {
    let mut children = vec![sized("N1", 80.0, 100.0), e, sized("N2", 80.0, 100.0)];
    children.extend(extra);
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
            children,
        ),
    )
}

/// The lower Screen: E in a horizontal scroll view 200 wide, scrolled 200
/// along, so E (120 to 280 in its strip) shows from -80 to 80: half out.
fn lower(e: (f32, f32)) -> Scene {
    lower_with(element(e))
}

fn lower_with(e: B) -> Scene {
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
                        vec![sized("M1", 120.0, 100.0), e, sized("M2", 120.0, 100.0)],
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

/// Today's slot path lays the element out as a unit: its own root at a
/// definite size. The held element's inside, relative to the element,
/// equals that unit's at every size the step passes through.
#[test]
fn probe_hold_inside_matches_a_unit_root_at_the_flight_size() {
    for screen in [&UPPER, &LOWER] {
        let scene = (screen.make)(screen.own);
        scene.advance(2);
        let e = handle(&scene, "E");
        assert!(LayoutComponent::begin_flight_hold(&e, LOWER_SIZE));
        for step in 0..=10 {
            let size = lerp(LOWER_SIZE, UPPER_SIZE, step as f32 / 10.0);
            LayoutComponent::set_flight_hold_size(&e, size);
            scene.advance(1);
            let unit = super::scene(size, element(size));
            unit.advance(2);
            assert_eq!(unit.bounds("E")[2..], [size.0, size.1]);
            for name in ["Fill", "Label"] {
                assert_eq!(scene.bounds(name), unit.bounds(name), "{name} at {size:?}");
            }
        }
        LayoutComponent::end_flight_hold(&e);
        scene.advance(1);
    }
}

// ---------------------------------------------------------------------------
// The placed lift, css_layout/placed_lift.rs.
//
// The lower Screen's E (dark blue here) is lifted: its own Screen's draw
// leaves it out, and the upper Screen draws it just before the upper copy
// (light blue), whole, at the upper Screen's place for it in paint order:
// over "Below" (absolute, z-index -1, overlapping E) and under "Over"
// (absolute, overlapping E).

use crate::css_layout::placed_lift::PlacedLift;
use crate::source::artboard::Artboard;
use crate::source::math::mat2d::Mat2D;
use crate::source::math::vec2d::Vec2D;

const E_LOW: u32 = 0xff00_0011;
const E_UP: u32 = 0xff00_0022;
const FILL_LOW: u32 = 0xff00_0033;
const BELOW: u32 = 0xff00_0044;
const OVER: u32 = 0xff00_0055;

fn painted_element(size: (f32, f32), own: u32, fill: Option<u32>) -> B {
    let mut e = element(size);
    e.paints = vec![Paint::Fill(own)];
    if let Some(fill) = fill {
        e.children[0].paints = vec![Paint::Fill(fill)];
    }
    e
}

fn absolute(name: &'static str, color: u32, at: (f32, f32), size: (f32, f32), z: Option<f32>) -> B {
    let inset = |kind: u16, v: f32| [U(kind, 1), F(kind + 1, v)];
    let mut props = [
        [U(POSITION, 2)].as_slice(),
        &inset(INSET_LEFT_KIND, at.0),
        &inset(INSET_TOP_KIND, at.1),
        &width(size.0),
        &height(size.1),
    ]
    .concat();
    if let Some(z) = z {
        props.extend([U(Z_INDEX_KIND, 1), F(Z_INDEX, z)]);
    }
    B {
        paints: vec![Paint::Fill(color)],
        ..css(name, props, vec![])
    }
}

fn lift_upper() -> Scene {
    upper_with(
        painted_element(UPPER_SIZE, E_UP, None),
        vec![
            absolute("Below", BELOW, (60.0, 0.0), (100.0, 100.0), Some(-1.0)),
            absolute("Over", OVER, (150.0, 20.0), (100.0, 40.0), None),
        ],
    )
}

fn lift_lower() -> Scene {
    lower_with(painted_element(LOWER_SIZE, E_LOW, Some(FILL_LOW)))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect4 {
    l: f32,
    t: f32,
    r: f32,
    b: f32,
}

#[derive(Debug)]
struct Drawn {
    at: usize,
    color: u32,
    rect: Rect4,
    clips: Vec<Rect4>,
}

fn rect_of(points: &[Vec2D]) -> Rect4 {
    let mut r = Rect4 {
        l: f32::MAX,
        t: f32::MAX,
        r: f32::MIN,
        b: f32::MIN,
    };
    for p in points {
        r.l = r.l.min(p.x);
        r.t = r.t.min(p.y);
        r.r = r.r.max(p.x);
        r.b = r.b.max(p.y);
    }
    r
}

fn points(line: &str, matrix: Mat2D) -> Vec<Vec2D> {
    let Some(list) = line
        .split("points=[")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
    else {
        return Vec::new();
    };
    if list.is_empty() {
        return Vec::new();
    }
    list.split("),(")
        .map(|point| {
            let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
            matrix * Vec2D::new(x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

/// Every colored path a draw records: where, over which rect, inside which
/// clips.
fn parse(lines: &[String]) -> Vec<Drawn> {
    let mut matrix = Mat2D::identity();
    let mut stack: Vec<(Mat2D, usize)> = Vec::new();
    let mut clips: Vec<Rect4> = Vec::new();
    let mut out = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if line == "save" {
            stack.push((matrix, clips.len()));
        } else if line == "restore" {
            let (saved, open) = stack.pop().expect("a save for each restore");
            matrix = saved;
            clips.truncate(open);
        } else if let Some(values) = line
            .strip_prefix("transform matrix=[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let mut local = Mat2D::identity();
            for (i, v) in values.split(',').enumerate() {
                local[i] = v.trim().parse().unwrap();
            }
            matrix *= local;
        } else if line.starts_with("clipPath ") {
            let pts = points(line, matrix);
            if !pts.is_empty() {
                clips.push(rect_of(&pts));
            }
        } else if line.starts_with("drawPath ") {
            let color = line
                .split(" paint=")
                .nth(1)
                .and_then(|paint| paint.split("color=0x").nth(1))
                .and_then(|hex| u32::from_str_radix(hex.get(..8)?, 16).ok());
            let pts = points(line, matrix);
            if let (Some(color), false) = (color, pts.is_empty()) {
                out.push(Drawn {
                    at,
                    color,
                    rect: rect_of(&pts),
                    clips: clips.clone(),
                });
            }
        }
    }
    assert!(stack.is_empty(), "every save restored");
    out
}

fn one(drawn: &[Drawn], color: u32, lines: &[String]) -> usize {
    let found: Vec<_> = drawn.iter().enumerate().filter(|(_, d)| d.color == color).collect();
    assert_eq!(found.len(), 1, "{color:08x} drawn once:\n{}", lines.join("\n"));
    found[0].0
}

/// A draw's lines without the factory's path creation lines, which only a
/// Screen's first draw records, and without path ids: a text reshaped at
/// another width during a step gets a new path with the same geometry.
fn steady(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !line.starts_with("make"))
        .map(|line| match line.split_once("path={id=") {
            Some((head, rest)) => {
                let tail = rest.split_once(',').map_or("", |(_, tail)| tail);
                format!("{head}path={{id=_,{tail}")
            }
            None => line,
        })
        .collect()
}

fn draws(lines: &[String]) -> usize {
    lines.iter().filter(|line| line.starts_with("drawPath")).count()
}

fn near(a: Rect4, b: Rect4) -> bool {
    let n = |x: f32, y: f32| (x - y).abs() < 0.05;
    n(a.l, b.l) && n(a.t, b.t) && n(a.r, b.r) && n(a.b, b.b)
}

fn r4(l: f32, t: f32, r: f32, b: f32) -> Rect4 {
    Rect4 { l, t, r, b }
}

/// The scenes draw as described: the lower copy clipped by its scroll view,
/// Below under the upper copy and Over over it.
#[test]
fn probe_lift_the_scenes_draw_as_described() {
    let (up, low) = (lift_upper(), lift_lower());
    up.advance(2);
    low.advance(2);
    let lines = low.draw_lines();
    let drawn = parse(&lines);
    let e = &drawn[one(&drawn, E_LOW, &lines)];
    assert!(near(e.rect, r4(-80.0, 0.0, 80.0, 120.0)), "{e:?}");
    assert!(
        e.clips.iter().any(|c| near(*c, r4(0.0, 0.0, 200.0, 150.0))),
        "the scroll view clips the lower copy: {e:?}"
    );
    let lines = up.draw_lines();
    let drawn = parse(&lines);
    let (below, e, over) = (
        one(&drawn, BELOW, &lines),
        one(&drawn, E_UP, &lines),
        one(&drawn, OVER, &lines),
    );
    assert!(below < e && e < over, "{}", lines.join("\n"));
    assert!(near(drawn[e].rect, r4(90.0, 0.0, 210.0, 100.0)), "{:?}", drawn[e]);
}

/// Lifted: the lower Screen leaves its copy out; the upper Screen draws it
/// whole (no scroll-view clip) between Below and its own copy, under Over;
/// a pose composed outside layout moves it; clearing the lift gives back
/// both resting draws, line for line.
#[test]
fn probe_lift_draws_the_lower_copy_whole_at_the_upper_copys_place() {
    let (up, low) = (lift_upper(), lift_lower());
    up.advance(2);
    low.advance(2);
    let (up_root, low_root) = (up.artboard.core_handle(), low.artboard.core_handle());
    let (e_up, e_low) = (handle(&up, "E"), handle(&low, "E"));
    let up_rest = up.draw_lines();
    let low_rest = low.draw_lines();
    let e_low_draws = {
        Artboard::set_lifts(&low_root, vec![e_low.clone()], vec![]);
        let skipped = low.draw_lines();
        let parsed = parse(&skipped);
        assert!(parsed.iter().all(|d| d.color != E_LOW && d.color != FILL_LOW));
        draws(&low_rest) - draws(&skipped)
    };
    assert!(e_low_draws >= 2, "E's own paint, Fill's and the label's");
    for (pose, want) in [
        (Mat2D::identity(), r4(-80.0, 0.0, 80.0, 120.0)),
        (Mat2D::from_translate(170.0, 0.0), r4(90.0, 0.0, 250.0, 120.0)),
    ] {
        Artboard::set_lifts(
            &up_root,
            vec![],
            vec![PlacedLift {
                anchor: e_up.clone(),
                row: low_root.clone(),
                element: e_low.clone(),
                transform: pose,
            }],
        );
        let lines = up.draw_lines();
        assert_eq!(draws(&lines), draws(&up_rest) + e_low_draws, "{}", lines.join("\n"));
        let drawn = parse(&lines);
        let (below, lifted, fill, own, over) = (
            one(&drawn, BELOW, &lines),
            one(&drawn, E_LOW, &lines),
            one(&drawn, FILL_LOW, &lines),
            one(&drawn, E_UP, &lines),
            one(&drawn, OVER, &lines),
        );
        assert!(below < lifted && lifted < fill && fill < own && own < over, "{}", lines.join("\n"));
        let e = &drawn[lifted];
        assert!(near(e.rect, want), "lifted copy at {:?}, want {want:?}", e.rect);
        // Drawn whole: under the upper Screen's clips at the upper copy's
        // place (its own bounds), none of the lower Screen's scroll view.
        assert_eq!(e.clips, drawn[own].clips, "drawn whole: {e:?}");
        assert_eq!(drawn[fill].clips, drawn[own].clips, "inside drawn whole");
        assert!(e.clips.iter().all(|c| near(*c, r4(0.0, 0.0, 400.0, 300.0))), "{e:?}");
    }
    Artboard::set_lifts(&up_root, vec![], vec![]);
    Artboard::set_lifts(&low_root, vec![], vec![]);
    assert_eq!(steady(up.draw_lines()), steady(up_rest), "upper Screen after the lift");
    assert_eq!(steady(low.draw_lines()), steady(low_rest), "lower Screen after the lift");
}

/// Hold and lift together, the lower copy flying to the upper size: on each
/// frame the upper Screen draws it whole at the flight size, its inside laid
/// out for that size, and the lower Screen's other boxes stay put.
#[test]
fn probe_lift_and_hold_together_across_a_step() {
    let (up, low) = (lift_upper(), lift_lower());
    up.advance(2);
    low.advance(2);
    let (up_root, low_root) = (up.artboard.core_handle(), low.artboard.core_handle());
    let (e_up, e_low) = (handle(&up, "E"), handle(&low, "E"));
    let up_rest = up.draw_lines();
    let low_rest = low.draw_lines();
    let rest = boxes(&low, &LOWER_NAMES);
    Artboard::set_lifts(&low_root, vec![e_low.clone()], vec![]);
    assert!(LayoutComponent::begin_flight_hold(&e_low, LOWER_SIZE));
    for step in 0..=10 {
        let p = step as f32 / 10.0;
        let size = lerp(LOWER_SIZE, UPPER_SIZE, p);
        LayoutComponent::set_flight_hold_size(&e_low, size);
        low.advance(1);
        // The flight's place, composed outside layout: from the lower copy's
        // drawn place (-80, 0) to the upper copy's (90, 0).
        let pose = Mat2D::from_translate(170.0 * p, 0.0);
        Artboard::set_lifts(
            &up_root,
            vec![],
            vec![PlacedLift {
                anchor: e_up.clone(),
                row: low_root.clone(),
                element: e_low.clone(),
                transform: pose,
            }],
        );
        let lines = up.draw_lines();
        let drawn = parse(&lines);
        let e = &drawn[one(&drawn, E_LOW, &lines)];
        let x = -80.0 + 170.0 * p;
        assert!(
            near(e.rect, r4(x, 0.0, x + size.0, size.1)),
            "p {p}: lifted copy at {:?}, size {size:?}",
            e.rect
        );
        let own = &drawn[one(&drawn, E_UP, &lines)];
        assert_eq!(e.clips, own.clips, "p {p}: drawn whole");
        for (name, at) in &rest {
            if matches!(name.as_str(), "E" | "Fill" | "Label") {
                continue;
            }
            assert_eq!(&low.bounds(name), at, "{name} at p {p}");
        }
    }
    LayoutComponent::end_flight_hold(&e_low);
    low.advance(1);
    Artboard::set_lifts(&up_root, vec![], vec![]);
    Artboard::set_lifts(&low_root, vec![], vec![]);
    assert_eq!(boxes(&low, &LOWER_NAMES), rest);
    assert_eq!(steady(up.draw_lines()), steady(up_rest));
    assert_eq!(steady(low.draw_lines()), steady(low_rest));
}

// ---------------------------------------------------------------------------
// Cost per frame, printed (debug build, so read the ratios, not the times).
// "relayout" is today's plain layout animation: E's authored size written
// each frame, the whole Screen solved. "unit" stands in for today's slot
// path's per-frame solve: E alone as a root at a definite size. "hold" is
// the probe: the flight size written each frame, the Screen solved with E
// held. No caches anywhere.
#[test]
fn probe_cost_per_frame() {
    use std::time::Instant;
    const N: usize = 400;
    fn time(label: &str, mut frame: impl FnMut(usize)) -> f64 {
        for i in 0..40 {
            frame(i);
        }
        let start = Instant::now();
        for i in 0..N {
            frame(i);
        }
        let per = start.elapsed().as_secs_f64() * 1e6 / N as f64;
        println!("PROBE COST {label}: {per:.1} us per frame");
        per
    }
    let wobble = |i: usize| (i % 40) as f32 / 40.0;
    for (name, make, own) in [("upper", upper as fn((f32, f32)) -> Scene, UPPER_SIZE), ("lower", lower, LOWER_SIZE)] {
        let s = make(own);
        s.advance(2);
        time(&format!("{name} rest"), |_| s.advance(1));
        let s = make(own);
        s.advance(2);
        let record = s.record("E");
        time(&format!("{name} relayout"), |i| {
            let size = lerp(LOWER_SIZE, UPPER_SIZE, wobble(i));
            assert!(CoreRegistry::set_double_handle(&record, WIDTH.into(), size.0));
            assert!(CoreRegistry::set_double_handle(&record, HEIGHT.into(), size.1));
            s.advance(1);
        });
        let s = make(own);
        s.advance(2);
        let e = s.named("E");
        assert!(LayoutComponent::begin_flight_hold(&e, own));
        time(&format!("{name} hold"), |i| {
            LayoutComponent::set_flight_hold_size(&e, lerp(LOWER_SIZE, UPPER_SIZE, wobble(i)));
            s.advance(1);
        });
        LayoutComponent::end_flight_hold(&e);
    }
    let unit = scene((400.0, 300.0), element(UPPER_SIZE));
    unit.advance(2);
    let record = unit.record("E");
    time("unit (E alone as a root)", |i| {
        let size = lerp(LOWER_SIZE, UPPER_SIZE, wobble(i));
        assert!(CoreRegistry::set_double_handle(&record, WIDTH.into(), size.0));
        assert!(CoreRegistry::set_double_handle(&record, HEIGHT.into(), size.1));
        unit.advance(1);
    });
    // Draws.
    let (up, low) = (lift_upper(), lift_lower());
    up.advance(2);
    low.advance(2);
    let draw_both = || {
        up.factory.borrow_mut().clear();
        low.factory.borrow_mut().clear();
        let _ = up.draw_lines();
        let _ = low.draw_lines();
    };
    time("draw both Screens, plain", |_| draw_both());
    let (up_root, low_root) = (up.artboard.core_handle(), low.artboard.core_handle());
    Artboard::set_lifts(&low_root, vec![handle(&low, "E")], vec![]);
    Artboard::set_lifts(
        &up_root,
        vec![],
        vec![PlacedLift {
            anchor: handle(&up, "E"),
            row: low_root.clone(),
            element: handle(&low, "E"),
            transform: Mat2D::identity(),
        }],
    );
    time("draw both Screens, lower copy lifted", |_| draw_both());
}
