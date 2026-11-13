//! PROBE (one shape for a Screen, probe 2): a component instance, a
//! transferable component and a list as the matched element, and the three
//! named gaps of probe 1 (the ancestor fade, a non-box anchor, the resting
//! reader), plus both copies lifted.
//!
//! The page is `css_paint_fixture(PaintCase::OneShape { .. })` (its doc
//! says what it holds). The hold, the lift and the carry are driven by hand,
//! as the first probe drove them: no step record names them yet. Oracles:
//! - a component's inside: the same component laid out today's unit way,
//!   its host's item record written at the flight size (the route
//!   `follow.rs` `set_flight_box` sizes a CSS flight slot by), in a second
//!   page advanced in lockstep;
//! - its own timeline: the same component's keyed value in that page;
//! - after a step: the resting boxes, node counts and draws read before it.
#![cfg(feature = "test-support")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::live_move_fixture::{self as fixture, PaintCase};
use nuxie_runtime::{
    css_layout::placed_lift::{PlacedLift, set_own_fade_only},
    live_move::{element_rest, move_live, refill_slot, screens, slot_pose},
    source::{
        artboard_component_list::ArtboardComponentList,
        generated::core_registry::CoreRegistry,
        layout_component::LayoutComponent,
        math::{mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
    },
    *,
};

use nuxie_runtime::css_layout::keys::{HEIGHT_KIND, PADDING_TOP, WIDTH_KIND};

struct Page {
    file: RuntimeFileHandle,
    factory: PersistentFactory<RecordingFactory>,
    root: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
}

impl Page {
    fn new(list: bool) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &fixture::css_paint_fixture(PaintCase::OneShape { list }),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
        let model = file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
            .unwrap();
        let machine = root.default_state_machine_handle().unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(model));
        machine.advance_and_apply(0.0);
        machine.advance_and_apply(0.0);
        Self {
            file,
            factory,
            root,
            machine,
        }
    }
    fn advance(&self, dt: f32) {
        self.machine.advance_and_apply(dt);
    }
    fn row(&self, name: &str) -> RuntimeArtboardInstanceHandle {
        screens(&self.root).unwrap()[name].clone()
    }
    fn slot(&self, row: &str, name: &str) -> CoreHandle {
        self.row(row)
            .with_artboard(|r| r.find_handle::<NestedArtboard>(name))
            .unwrap_or_else(|| panic!("no slot {name} on {row}"))
    }
    fn el(&self, row: &str, name: &str) -> CoreHandle {
        self.row(row)
            .with_artboard(|r| r.find_handle::<LayoutComponent>(name))
            .unwrap_or_else(|| panic!("no box {name} on {row}"))
    }
    /// The card's instance on a Screen, and its root.
    fn card(&self, row: &str) -> (RuntimeArtboardInstanceHandle, CoreHandle) {
        let instance = child(&self.slot(row, "card")).expect("a mounted card");
        let root = instance.core_handle();
        (instance, root)
    }
    fn lines(&self, row: &str) -> Vec<String> {
        self.factory.borrow_mut().clear();
        let mut renderer = self.factory.borrow().make_renderer();
        self.row(row).draw(&mut renderer);
        self.factory
            .borrow()
            .stream()
            .lines()
            .filter(|line| !line.starts_with("make"))
            .map(|line| match line.split_once("path={id=") {
                Some((head, rest)) => {
                    let tail = rest.split_once(',').map_or("", |(_, tail)| tail);
                    format!("{head}path={{id=_,{tail}")
                }
                None => line.to_owned(),
            })
            .collect()
    }
    fn draws(&self, row: &str) -> (Vec<Drawn>, Vec<String>) {
        let lines = self.lines(row);
        (parse(&lines), lines)
    }
}

fn child(slot: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
    slot.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
}

/// A box's (or an artboard root's) solved box: x, y, width, height.
fn bounds(h: &CoreHandle) -> [f32; 4] {
    h.with(|o| {
        let l = o.as_layout_component().expect("a layout box");
        [l.layout_x(), l.layout_y(), l.layout_width(), l.layout_height()]
    })
    .unwrap()
}

fn record(h: &CoreHandle) -> CoreHandle {
    h.with(|o| o.as_layout_component().and_then(|l| l.css_record()))
        .flatten()
        .expect("a CSS record")
}

fn padding(card_root: &CoreHandle) -> f32 {
    CoreRegistry::get_double_handle(&record(card_root), i32::from(PADDING_TOP)).unwrap()
}

/// The Screen's layout tree: (Taffy nodes, nodes reachable from the root,
/// placeholders), read at the Screen or, when the Screen has none, the page.
fn counts(page: &Page, row: &str) -> (usize, usize, usize) {
    LayoutComponent::layout_tree_counts(&page.row(row).core_handle())
        .or_else(|| LayoutComponent::layout_tree_counts(&page.root.core_handle()))
        .expect("a layout tree")
}

/// Today's unit path: the host's item record at `size`, as `set_flight_box`
/// sizes a CSS flight slot.
fn size_host(slot: &CoreHandle, size: (f32, f32)) {
    let record = slot
        .with(|o| {
            o.as_container_component().and_then(|c| {
                c.children()
                    .iter()
                    .find(|ch| ch.is_type_of(nuxie_runtime::css_layout::keys::LAYOUT_CSS_STYLE))
                    .cloned()
            })
        })
        .flatten()
        .expect("the host's item record");
    for (kind, value) in [(WIDTH_KIND, size.0), (HEIGHT_KIND, size.1)] {
        CoreRegistry::set_uint_handle(&record, i32::from(kind), 1);
        CoreRegistry::set_double_handle(&record, i32::from(kind + 1), value);
        CoreRegistry::set_double_handle(&record, i32::from(kind + 2), 0.0);
    }
}

fn lerp(a: (f32, f32), b: (f32, f32), p: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * p, a.1 + (b.1 - a.1) * p)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect4 {
    l: f32,
    t: f32,
    r: f32,
    b: f32,
}
fn r4(l: f32, t: f32, r: f32, b: f32) -> Rect4 {
    Rect4 { l, t, r, b }
}
fn near(a: Rect4, b: Rect4) -> bool {
    let n = |x: f32, y: f32| (x - y).abs() < 0.05;
    n(a.l, b.l) && n(a.t, b.t) && n(a.r, b.r) && n(a.b, b.b)
}

#[derive(Clone, Debug)]
struct Drawn {
    at: usize,
    /// ARGB, as the paint holds it: alpha carries the baked opacity.
    color: u32,
    rect: Rect4,
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

fn parse(lines: &[String]) -> Vec<Drawn> {
    let mut matrix = Mat2D::identity();
    let mut stack: Vec<Mat2D> = Vec::new();
    let mut out = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if line == "save" {
            stack.push(matrix);
        } else if line == "restore" {
            matrix = stack.pop().expect("a save for each restore");
        } else if let Some(values) = line
            .strip_prefix("transform matrix=[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let mut local = Mat2D::identity();
            for (i, v) in values.split(',').enumerate() {
                local[i] = v.trim().parse().unwrap();
            }
            matrix *= local;
        } else if line.starts_with("drawPath ") {
            let color = line
                .split(" paint=")
                .nth(1)
                .and_then(|paint| paint.split("color=0x").nth(1))
                .and_then(|hex| u32::from_str_radix(hex.get(..8)?, 16).ok());
            let pts = points(line, matrix);
            if let (Some(color), false) = (color, pts.is_empty()) {
                let (mut l, mut t, mut r, mut b) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for p in &pts {
                    l = l.min(p.x);
                    t = t.min(p.y);
                    r = r.max(p.x);
                    b = b.max(p.y);
                }
                out.push(Drawn {
                    at,
                    color,
                    rect: r4(l, t, r, b),
                });
            }
        }
    }
    out
}

fn rgb(name: &str) -> u32 {
    fixture::paint_color(name) & 0x00ff_ffff
}

/// Every draw of `name`'s color, in stream order.
fn all<'a>(drawn: &'a [Drawn], name: &str) -> Vec<&'a Drawn> {
    drawn.iter().filter(|d| d.color & 0x00ff_ffff == rgb(name)).collect()
}

fn one<'a>(drawn: &'a [Drawn], name: &str, lines: &[String]) -> &'a Drawn {
    let found = all(drawn, name);
    assert_eq!(found.len(), 1, "{name} drawn once:\n{}", lines.join("\n"));
    found[0]
}

fn drawpaths(lines: &[String]) -> usize {
    lines.iter().filter(|line| line.starts_with("drawPath")).count()
}

// Q1 ------------------------------------------------------------------------

/// The page rests as its doc says: both cards placed in their rows, the
/// card's child 10 from its right and bottom, and its own timeline playing.
#[test]
fn probe2_the_page_rests_as_described() {
    let page = Page::new(false);
    page.advance(0.1);
    assert_eq!(bounds(&page.el("B", "n1")), [0.0, 0.0, 80.0, 100.0]);
    let (_, b) = page.card("B");
    assert_eq!(bounds(&b), [90.0, 0.0, 120.0, 80.0]);
    assert_eq!(bounds(&page.el("B", "n2")), [220.0, 0.0, 80.0, 100.0]);
    let (_, a) = page.card("A");
    assert_eq!(bounds(&a), [110.0, 0.0, 160.0, 100.0]);
    assert_eq!(bounds(&page.el("A", "m2")), [280.0, 0.0, 100.0, 100.0]);
    let child_b = child(&page.slot("B", "card"))
        .unwrap()
        .with_artboard(|c| c.find_handle::<LayoutComponent>("child"))
        .unwrap();
    assert_eq!(bounds(&child_b), [120.0 - 40.0, 80.0 - 30.0, 30.0, 20.0]);
    let p = padding(&b);
    assert!(p > 0.0 && p < 30.0, "the card's timeline plays: {p}");
    eprintln!("probe2 counts A {:?} B {:?}", counts(&page, "A"), counts(&page, "B"));
}

/// The component instance as the matched element. Its instance's root is
/// held in its host's place. On every frame, forward on the upper copy and
/// on the lower copy: its row's other boxes rest; it keeps its resting
/// place at the flight size; its inside (the absolutely placed child) and
/// its own timeline's value equal today's unit path (the host's record at
/// that size) in a page advanced in lockstep; one placeholder. After: the
/// resting tree, boxes and node counts.
#[test]
fn probe2_component_hold_lays_out_inside_at_the_flight_size_and_keeps_playing() {
    for (row, others, from, to) in [
        ("B", ["n1", "n2", "row"], (120.0, 80.0), (160.0, 100.0)),
        ("A", ["m1", "m2", "fade"], (160.0, 100.0), (120.0, 80.0)),
    ] {
        let held = Page::new(false);
        let today = Page::new(false);
        held.advance(0.1);
        today.advance(0.1);
        let rest: Vec<_> = others.iter().map(|n| bounds(&held.el(row, n))).collect();
        let rest_counts = counts(&held, row);
        let (instance, card) = held.card(row);
        let card_rest = bounds(&card);
        let (today_instance, today_card) = today.card(row);
        let child_of = |i: &RuntimeArtboardInstanceHandle| {
            i.with_artboard(|c| c.find_handle::<LayoutComponent>("child"))
                .unwrap()
        };
        assert!(
            LayoutComponent::begin_flight_hold(&card, from),
            "a component instance's root has a layout parent to hold"
        );
        let mut paddings = Vec::new();
        for step in 0..=10 {
            let p = step as f32 / 10.0;
            let size = lerp(from, to, p);
            LayoutComponent::set_flight_hold_size(&card, size);
            size_host(&today.slot(row, "card"), size);
            held.advance(0.05);
            today.advance(0.05);
            for (name, at) in others.iter().zip(&rest) {
                assert_eq!(&bounds(&held.el(row, name)), at, "{row}: {name} moved at {p}");
            }
            assert_eq!(
                bounds(&card),
                [card_rest[0], card_rest[1], size.0, size.1],
                "{row}: the card at {p}"
            );
            assert_eq!(
                LayoutComponent::flight_hold_rest(&card).map(|r| [
                    r.left(),
                    r.top(),
                    r.width(),
                    r.height()
                ]),
                Some(card_rest),
                "{row}: the held rest at {p}"
            );
            // Its inside equals today's unit path at the same size.
            assert_eq!(bounds(&today_card)[2..], [size.0, size.1], "{row}: oracle size");
            assert_eq!(
                bounds(&child_of(&instance)),
                bounds(&child_of(&today_instance)),
                "{row}: the child at {p}"
            );
            // Its own timeline keeps playing, as it does there.
            assert_eq!(padding(&card), padding(&today_card), "{row}: padding at {p}");
            paddings.push(padding(&card));
            assert_eq!(counts(&held, row).2, 1, "{row}: one placeholder at {p}");
        }
        assert!(
            paddings.windows(2).all(|w| w[1] > w[0]),
            "{row}: the timeline advanced every frame: {paddings:?}"
        );
        LayoutComponent::end_flight_hold(&card);
        held.advance(0.0);
        let after: Vec<_> = others.iter().map(|n| bounds(&held.el(row, n))).collect();
        assert_eq!(after, rest, "{row}: after");
        assert_eq!(bounds(&card), card_rest, "{row}: the card after");
        assert_eq!(counts(&held, row), rest_counts, "{row}: the resting tree after");
        assert!(LayoutComponent::flight_hold_rest(&card).is_none());
    }
}

/// The lift with a component instance: the lower Screen leaves its card
/// out; the upper Screen draws it whole (its fill and its child, from the
/// nested artboard's one entry) just before its own card, under "over",
/// after n1. Cleared, both Screens draw their resting draws line for line.
#[test]
fn probe2_component_lift_draws_the_lower_card_as_one_entry() {
    let page = Page::new(false);
    page.advance(0.1);
    let (a_row, b_row) = (page.row("A").core_handle(), page.row("B").core_handle());
    let (a_card, b_card) = (page.slot("A", "card"), page.slot("B", "card"));
    let a_rest = page.lines("A");
    let b_rest = page.lines("B");
    Artboard::set_lifts(&a_row, vec![a_card.clone()], vec![]);
    let (a_drawn, a_lines) = page.draws("A");
    assert!(all(&a_drawn, "card").is_empty(), "{}", a_lines.join("\n"));
    let lifted_paths = drawpaths(&a_rest) - drawpaths(&a_lines);
    assert_eq!(lifted_paths, 2, "the card's fill and its child");
    Artboard::set_lifts(
        &b_row,
        vec![],
        vec![PlacedLift {
            anchor: b_card.clone(),
            row: a_row.clone(),
            element: a_card.clone(),
            transform: Mat2D::identity(),
        }],
    );
    let (drawn, lines) = page.draws("B");
    assert_eq!(drawpaths(&lines), drawpaths(&b_rest) + lifted_paths);
    let cards = all(&drawn, "card");
    let children = all(&drawn, "card child");
    assert_eq!((cards.len(), children.len()), (2, 2), "{}", lines.join("\n"));
    // The lower card first, at its own place on A, whole; then B's own.
    assert!(near(cards[0].rect, r4(110.0, 0.0, 270.0, 100.0)), "{:?}", cards[0]);
    assert!(near(cards[1].rect, r4(90.0, 0.0, 210.0, 80.0)), "{:?}", cards[1]);
    assert!(near(children[0].rect, r4(230.0, 70.0, 260.0, 90.0)), "{:?}", children[0]);
    let n1 = one(&drawn, "n1", &lines).at;
    let over = one(&drawn, "over", &lines).at;
    assert!(n1 < cards[0].at && cards[0].at < children[0].at);
    assert!(children[0].at < cards[1].at && cards[1].at < over);
    Artboard::set_lifts(&a_row, vec![], vec![]);
    Artboard::set_lifts(&b_row, vec![], vec![]);
    assert_eq!(page.lines("A"), a_rest, "A after");
    assert_eq!(page.lines("B"), b_rest, "B after");
}

// Q2 ------------------------------------------------------------------------

/// A transferable component in place: one instance carried straight from
/// the leaving placement into the arriving one at take-off (no flight slot),
/// held there at the arriving placement's resting place, read before the
/// carry, and flown from the leaving size to its own. On every frame: the
/// arriving row's other boxes rest, the leaving row's neighbors rest (its
/// emptied placement's frozen leaf), the carried card is at its arriving
/// place and the flight size, and its own timeline keeps playing on from
/// where it was (equal to an uncarried card in a page in lockstep). At
/// landing the leaving placement is refilled; both rows are their resting
/// trees, and the carried instance rests in the arriving placement.
#[test]
fn probe2_transfer_carries_one_instance_into_the_arriving_placement_and_holds_it() {
    let page = Page::new(false);
    let today = Page::new(false);
    page.advance(0.3);
    today.advance(0.3);
    let (a_slot, b_slot) = (page.slot("A", "card"), page.slot("B", "card"));
    let a_rest: Vec<_> = ["m1", "m2", "fade"].iter().map(|n| bounds(&page.el("A", n))).collect();
    let b_rest: Vec<_> = ["n1", "n2", "row"].iter().map(|n| bounds(&page.el("B", n))).collect();
    let (a_counts, b_counts) = (counts(&page, "A"), counts(&page, "B"));
    let (carried, carried_root) = page.card("A");
    let (_, b_own) = page.card("B");
    let b_place = LayoutComponent::resting_place(&b_own).unwrap();
    let b_card_rest = bounds(&b_own);
    let (_, today_a) = today.card("A");
    let before = padding(&carried_root);
    assert_eq!(before, padding(&today_a));
    // Take-off.
    move_live(&a_slot, &b_slot).expect("two ordinary placements carry");
    assert!(child(&a_slot).is_none());
    assert_eq!(child(&b_slot).unwrap().core_handle(), carried.core_handle());
    assert!(
        LayoutComponent::begin_flight_hold_from(&carried_root, Some(b_place), (160.0, 100.0)),
        "the carried root is held in its new placement"
    );
    for step in 0..=10 {
        let p = step as f32 / 10.0;
        let size = lerp((160.0, 100.0), (120.0, 80.0), p);
        LayoutComponent::set_flight_hold_size(&carried_root, size);
        page.advance(0.05);
        today.advance(0.05);
        let a_now: Vec<_> = ["m1", "m2", "fade"].iter().map(|n| bounds(&page.el("A", n))).collect();
        let b_now: Vec<_> = ["n1", "n2", "row"].iter().map(|n| bounds(&page.el("B", n))).collect();
        assert_eq!(a_now, a_rest, "A's neighbors at {p}");
        assert_eq!(b_now, b_rest, "B's neighbors at {p}");
        assert_eq!(
            bounds(&carried_root),
            [b_card_rest[0], b_card_rest[1], size.0, size.1],
            "the carried card at {p}"
        );
        assert_eq!(
            padding(&carried_root),
            padding(&today_a),
            "its timeline at {p}"
        );
    }
    assert!(padding(&carried_root) > before, "it played on");
    // Landing.
    LayoutComponent::end_flight_hold(&carried_root);
    refill_slot(&a_slot).expect("the leaving placement is refilled");
    page.advance(0.0);
    let own = child(&a_slot).expect("A has its own card again");
    assert_ne!(own.core_handle(), carried.core_handle());
    assert_eq!(child(&b_slot).unwrap().core_handle(), carried.core_handle());
    assert_eq!(bounds(&carried_root), b_card_rest, "the carried card rests in B");
    assert_eq!(bounds(&own.core_handle()), [110.0, 0.0, 160.0, 100.0]);
    let a_now: Vec<_> = ["m1", "m2", "fade"].iter().map(|n| bounds(&page.el("A", n))).collect();
    let b_now: Vec<_> = ["n1", "n2", "row"].iter().map(|n| bounds(&page.el("B", n))).collect();
    assert_eq!((a_now, b_now), (a_rest, b_rest));
    assert_eq!((counts(&page, "A"), counts(&page, "B")), (a_counts, b_counts));
    assert_eq!(counts(&page, "B").2, 0);
    // It keeps playing after landing.
    let landed = padding(&carried_root);
    page.advance(0.1);
    today.advance(0.1);
    assert_eq!(padding(&carried_root), padding(&today_a));
    assert!(padding(&carried_root) > landed);
}

/// The same carry cancelled halfway: the instance goes home, and the
/// arriving placement, whose own instance the take-off discarded, is
/// refilled. Both rows rest as before.
#[test]
fn probe2_transfer_cancelled_halfway_goes_home() {
    let page = Page::new(false);
    page.advance(0.3);
    let (a_slot, b_slot) = (page.slot("A", "card"), page.slot("B", "card"));
    let (a_counts, b_counts) = (counts(&page, "A"), counts(&page, "B"));
    let (carried, carried_root) = page.card("A");
    let (_, b_own) = page.card("B");
    let place = LayoutComponent::resting_place(&b_own).unwrap();
    move_live(&a_slot, &b_slot).unwrap();
    assert!(LayoutComponent::begin_flight_hold_from(&carried_root, Some(place), (160.0, 100.0)));
    for step in 0..5 {
        LayoutComponent::set_flight_hold_size(
            &carried_root,
            lerp((160.0, 100.0), (120.0, 80.0), step as f32 / 10.0),
        );
        page.advance(0.05);
    }
    LayoutComponent::end_flight_hold(&carried_root);
    move_live(&b_slot, &a_slot).expect("home");
    refill_slot(&b_slot).expect("B's own copy again");
    page.advance(0.0);
    assert_eq!(child(&a_slot).unwrap().core_handle(), carried.core_handle());
    assert_eq!(bounds(&carried_root), [110.0, 0.0, 160.0, 100.0]);
    assert_eq!(bounds(&child(&b_slot).unwrap().core_handle()), [90.0, 0.0, 120.0, 80.0]);
    assert_eq!((counts(&page, "A"), counts(&page, "B")), (a_counts, b_counts));
}

/// The carry's state rules hold between two placements directly, with no
/// flight slot between: typed text, focus and a playing animation survive
/// take-off, the flight and landing, and the leaving placement is refilled
/// with a fresh copy at its entry state (`fixture(2)`: A's slot holds the
/// mascot with a field "edit" and a "bob" keying y 0 to 100 over a second;
/// B's first slot is its arriving placement).
#[test]
fn probe2_transfer_keeps_typed_text_focus_and_playing_state_between_placements() {
    use nuxie_runtime::source::{focus_data::FocusData, node::Node, text::text_input::TextInput};
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::fixture(2),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let vm = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(vm));
    machine.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    let a_slot = rows["A"].with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
    let b_slot = rows["B"].with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
    let mascot = child(&a_slot).unwrap();
    let definition = mascot.with_artboard(|b| b.artboard_source_handle());
    let keyed = mascot.with_artboard(|b| b.find_handle::<Node>("keyed")).unwrap();
    let y = || CoreRegistry::get_double_handle(&keyed, 14).unwrap();
    let text = mascot.with_artboard(|b| b.find_handle::<TextInput>("edit")).unwrap();
    assert!(text.with_downcast_mut::<TextInput, _>(|t| t.text_input("hello mascot")).unwrap());
    let node = mascot.with_artboard(|b| b.find_all_handles::<FocusData>())[0]
        .with_downcast::<FocusData, _>(|f| f.existing_focus_node())
        .flatten()
        .unwrap();
    let manager = mascot.with_artboard(|b| b.focus_manager()).unwrap();
    manager.with_focus_manager_mut(|m| m.set_focus(node.clone()));
    machine.advance_and_apply(0.3);
    assert!((y() - 30.0).abs() < 0.01);
    move_live(&a_slot, &b_slot).expect("placement to placement");
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
    for step in 1..=5 {
        machine.advance_and_apply(0.1);
        assert!((y() - (30.0 + 10.0 * step as f32)).abs() < 0.01, "bob at {step}: {}", y());
    }
    refill_slot(&a_slot).expect("refill");
    machine.advance_and_apply(0.0);
    assert_eq!(child(&b_slot).unwrap().core_handle(), mascot.core_handle());
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
    assert!(text.with_downcast::<TextInput, _>(|t| t.is_focused()).unwrap());
    assert_eq!(text.with_downcast::<TextInput, _>(|t| t.text().to_owned()).unwrap(), "hello mascot");
    let own = child(&a_slot).unwrap();
    assert_ne!(own.core_handle(), mascot.core_handle());
    assert_eq!(own.with_artboard(|b| b.artboard_source_handle()), definition);
    let own_keyed = own.with_artboard(|b| b.find_handle::<Node>("keyed")).unwrap();
    assert!(CoreRegistry::get_double_handle(&own_keyed, 14).unwrap().abs() < 0.01);
    machine.advance_and_apply(0.1);
    assert!((y() - 90.0).abs() < 0.01);
}

// Q3 ------------------------------------------------------------------------

fn rows_list(page: &Page) -> (CoreHandle, CoreHandle) {
    let list = page
        .row("B")
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("rows"))
        .unwrap();
    let model = page
        .row("B")
        .data_context()
        .unwrap()
        .with_context(|c| c.main_view_model_instance())
        .unwrap();
    let items = model
        .with_downcast::<nuxie_runtime::source::viewmodel::viewmodel_instance::ViewModelInstance, _>(
            |vm| vm.property_value_named("rows"),
        )
        .flatten()
        .expect("B's rows");
    (list, items)
}

fn row_count(list: &CoreHandle) -> usize {
    list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
        .unwrap()
}

/// A list as the matched element: "list box" is held, its rows (a list of
/// B's model) reflow at the flight size, and rows are added and removed
/// under it during the step. On every frame its row's other boxes rest and
/// one placeholder stands; each row is laid out in the list box at the
/// flight width. After: the resting tree with the rows the list now holds,
/// equal to a fresh page given the same rows.
#[test]
fn probe2_list_hold_with_rows_added_and_removed_during_the_step() {
    use nuxie_runtime::source::{
        core::CoreArena,
        viewmodel::{
            viewmodel_instance_list::ViewModelInstanceList,
            viewmodel_instance_list_item::ViewModelInstanceListItem,
        },
    };
    let append = |page: &Page, arena: &CoreArena, n: usize| {
        let (_, items) = rows_list(page);
        let model = items
            .with_downcast::<ViewModelInstanceList, _>(|l| l.list_items()[0].clone())
            .unwrap()
            .with_downcast::<ViewModelInstanceListItem, _>(|i| i.view_model_instance())
            .flatten()
            .unwrap()
            .with_downcast::<nuxie_runtime::source::viewmodel::viewmodel_instance::ViewModelInstance, _>(
                |vm| vm.get_view_model(),
            )
            .flatten()
            .unwrap();
        for _ in 0..n {
            let vm = page
                .file
                .with_file(|f| f.create_default_view_model_instance(model.clone()))
                .unwrap();
            let item = arena.insert(ViewModelInstanceListItem::default());
            item.with_downcast_mut::<ViewModelInstanceListItem, _>(|i| {
                i.set_view_model_instance(Some(vm))
            });
            items.with_downcast_mut::<ViewModelInstanceList, _>(|l| l.add_item(item));
        }
    };
    let remove = |page: &Page, n: usize| {
        let (_, items) = rows_list(page);
        for _ in 0..n {
            items.with_downcast_mut::<ViewModelInstanceList, _>(|l| l.pop());
        }
    };
    let arena = CoreArena::default();
    let page = Page::new(true);
    page.advance(1.2);
    append(&page, &arena, 2);
    page.advance(0.0);
    page.advance(1.2);
    let (list, _) = rows_list(&page);
    assert_eq!(row_count(&list), 3);
    let others = ["n1", "n2", "row"];
    let rest: Vec<_> = others.iter().map(|n| bounds(&page.el("B", n))).collect();
    let box_ = page.el("B", "list box");
    let box_rest = bounds(&box_);
    assert_eq!(box_rest, [90.0, 0.0, 200.0, 150.0]);
    let rest_counts = counts(&page, "B");
    assert!(LayoutComponent::begin_flight_hold(&box_, (200.0, 150.0)));
    let mut seen = Vec::new();
    for step in 0..=10 {
        let p = step as f32 / 10.0;
        let size = lerp((200.0, 150.0), (120.0, 220.0), p);
        LayoutComponent::set_flight_hold_size(&box_, size);
        match step {
            3 => append(&page, &arena, 3),
            6 => remove(&page, 4),
            _ => {}
        }
        page.advance(0.05);
        for (name, at) in others.iter().zip(&rest) {
            assert_eq!(&bounds(&page.el("B", name)), at, "{name} moved at {p}");
        }
        assert_eq!(bounds(&box_), [box_rest[0], box_rest[1], size.0, size.1], "list box at {p}");
        let (_, _, held) = counts(&page, "B");
        assert_eq!(held, 1, "one placeholder at {p}");
        let n = row_count(&list);
        // Rows stack in the column; a row added during the step plays its
        // own height timeline from 0, the rows from before rest at 40.
        let mut y = 0.0;
        for index in 0..n {
            let row = list
                .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index as i32))
                .flatten()
                .unwrap();
            let (x, ry, w, h) =
                row.with_artboard(|r| (r.layout_x(), r.layout_y(), r.layout_width(), r.layout_height()));
            assert_eq!((x, ry), (0.0, y), "row {index} at {p}");
            if step < 3 || index < 3 {
                assert_eq!(h, 40.0, "row {index}'s height at {p}");
            } else {
                assert!((0.0..=40.0).contains(&h), "row {index}'s height at {p}: {h}");
            }
            y += h;
            assert_eq!(w, 200.0, "row {index}'s own width at {p}");
        }
        seen.push((n, counts(&page, "B").1));
    }
    eprintln!("probe2 list rows and reachable nodes per frame: {seen:?}");
    assert_eq!(seen[0].0, 3);
    assert_eq!(seen[3].0, 6);
    assert_eq!(seen[6].0, 2);
    LayoutComponent::end_flight_hold(&box_);
    page.advance(0.0);
    page.advance(1.2);
    assert_eq!(bounds(&box_), box_rest);
    let after: Vec<_> = others.iter().map(|n| bounds(&page.el("B", n))).collect();
    assert_eq!(after, rest);
    let (total, reachable, held) = counts(&page, "B");
    assert_eq!(held, 0);
    // A fresh page with the same two rows.
    let fresh = Page::new(true);
    fresh.advance(1.2);
    append(&fresh, &arena, 1);
    fresh.advance(0.0);
    fresh.advance(1.2);
    assert_eq!(row_count(&rows_list(&fresh).0), 2);
    assert_eq!(counts(&fresh, "B").1, reachable, "reachable nodes equal a fresh page's");
    eprintln!(
        "probe2 list counts: rest {rest_counts:?}, after {:?}, fresh {:?}",
        (total, reachable, held),
        counts(&fresh, "B")
    );
}

/// A list that virtualizes (upstream's `component_list_virtualized.riv`,
/// Rive-made: its rows are realized by its scroll window). Its viewport is
/// held and grown and shrunk: rows are realized and released under the
/// hold. The viewport's siblings rest, one placeholder stands, and after
/// the hold the tree and the realized rows are as before.
#[test]
fn probe2_virtualized_list_realizes_and_releases_rows_under_a_hold() {
    let path = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR"))
        .join("tests/unit_tests/assets/component_list_virtualized.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &std::fs::read(path).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let main = file.with_file(|f| f.artboard_named("Main")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(main.core_handle()))
        .unwrap();
    main.bind_view_model_instance(Some(model));
    main.advance_default(0.0);
    main.advance_default(0.0);
    let list = main
        .with_artboard(|m| m.find_handle::<ArtboardComponentList>("List"))
        .unwrap();
    assert!(list.with_downcast::<ArtboardComponentList, _>(|l| l.virtualization_enabled()).unwrap());
    // The boxes up from the list: its content, the viewport, and on.
    let mut chain = Vec::new();
    let mut at = list.with(|o| o.component_parent_handle()).flatten();
    while let Some(h) = at {
        let is_box = h.with(|o| o.as_layout_component().is_some()).unwrap_or(false);
        let name = h.with(|o| o.as_component().map(|c| c.name().to_owned())).flatten();
        eprintln!("probe2 virtualized chain: {name:?} box={is_box} {:?}", is_box.then(|| bounds(&h)));
        if is_box {
            chain.push(h.clone());
        }
        at = h.with(|o| o.component_parent_handle()).flatten();
    }
    let realized = || {
        list.with_downcast::<ArtboardComponentList, _>(|l| {
            (0..l.artboard_count())
                .filter(|&i| l.artboard_instance(i as i32).is_some())
                .count()
        })
        .unwrap()
    };
    let rest_realized = realized();
    let rest_counts = LayoutComponent::layout_tree_counts(&main.core_handle());
    eprintln!("probe2 virtualized: realized {rest_realized}, counts {rest_counts:?}");
    // The viewport is the content's parent box.
    let viewport = chain[1].clone();
    let siblings: Vec<_> = viewport
        .with(|o| o.component_parent_handle())
        .flatten()
        .and_then(|p| p.with(|o| o.as_container_component().map(|c| c.children().to_vec())).flatten())
        .unwrap_or_default()
        .into_iter()
        .filter(|h| h != &viewport && h.with(|o| o.as_layout_component().is_some()).unwrap_or(false))
        .collect();
    let siblings_rest: Vec<_> = siblings.iter().map(bounds).collect();
    let rest = bounds(&viewport);
    assert!(LayoutComponent::begin_flight_hold(&viewport, (rest[2], rest[3])));
    let mut seen = Vec::new();
    for scale in [1.0, 0.5, 0.25, 1.0, 2.0, 3.0, 1.0] {
        // The list scrolls across: its window is the viewport's width.
        let size = (rest[2] * scale, rest[3]);
        LayoutComponent::set_flight_hold_size(&viewport, size);
        main.advance_default(0.0);
        main.advance_default(0.0);
        let now: Vec<_> = siblings.iter().map(bounds).collect();
        assert_eq!(now, siblings_rest, "siblings at {scale}");
        assert_eq!(bounds(&viewport), [rest[0], rest[1], size.0, size.1], "viewport at {scale}");
        let counts = LayoutComponent::layout_tree_counts(&main.core_handle()).unwrap();
        assert_eq!(counts.2, 1, "one placeholder at {scale}");
        seen.push((scale, realized(), counts.1));
    }
    eprintln!("probe2 virtualized (scale, realized rows, reachable nodes): {seen:?}");
    assert!(
        seen.iter().any(|s| s.1 != rest_realized),
        "the hold's size changes what the window realizes"
    );
    LayoutComponent::end_flight_hold(&viewport);
    main.advance_default(0.0);
    main.advance_default(0.0);
    assert_eq!(bounds(&viewport), rest);
    assert_eq!(realized(), rest_realized);
    let after_counts = LayoutComponent::layout_tree_counts(&main.core_handle());
    // Control, no hold: the same widths reached by sizing Main itself (the
    // viewport fills it), in a second file.
    let mut factory2 = PersistentFactory::new(RecordingFactory::new());
    let file2 = File::import(
        &std::fs::read(
            std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap())
                .join("tests/unit_tests/assets/component_list_virtualized.riv"),
        )
        .unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory2).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let main2 = file2.with_file(|f| f.artboard_named("Main")).unwrap();
    let model2 = file2
        .with_file(|f| f.create_default_view_model_instance_for_artboard(main2.core_handle()))
        .unwrap();
    main2.bind_view_model_instance(Some(model2));
    main2.advance_default(0.0);
    main2.advance_default(0.0);
    let list2 = main2
        .with_artboard(|m| m.find_handle::<ArtboardComponentList>("List"))
        .unwrap();
    let realized2 = || {
        list2
            .with_downcast::<ArtboardComponentList, _>(|l| {
                (0..l.artboard_count())
                    .filter(|&i| l.artboard_instance(i as i32).is_some())
                    .count()
            })
            .unwrap()
    };
    let control_rest = LayoutComponent::layout_tree_counts(&main2.core_handle());
    let mut control = Vec::new();
    for scale in [1.0, 0.5, 0.25, 1.0, 2.0, 3.0, 1.0] {
        main2.set_size(rest[2] * scale, rest[3]);
        main2.advance_default(0.0);
        main2.advance_default(0.0);
        let counts = LayoutComponent::layout_tree_counts(&main2.core_handle()).unwrap();
        control.push((scale, realized2(), counts.1));
    }
    eprintln!(
        "probe2 virtualized: rest {rest_counts:?}, after the hold {after_counts:?}; control without a hold: rest {control_rest:?}, {control:?}"
    );
    // The hold leaves what the list leaves without one.
    assert_eq!(
        after_counts.map(|c| c.1),
        Some(control.last().unwrap().2),
        "reachable nodes after equal the control's"
    );
    assert_eq!(seen.iter().map(|s| s.1).collect::<Vec<_>>(), control.iter().map(|s| s.1).collect::<Vec<_>>());
}

// Q4 ------------------------------------------------------------------------

/// The undo of an ancestor's fade. A's "fade" row is at 0.5, baked into each
/// paint under it. Lifted with its own fade only, A's card and A's m1 draw at
/// their own opacity in B's draw; without it they carry the row's half.
/// Cleared, A draws as it rested.
#[test]
fn probe2_a_fading_ancestor_does_not_fade_the_lifted_copy() {
    let page = Page::new(false);
    page.advance(0.1);
    let (a_row, b_row) = (page.row("A").core_handle(), page.row("B").core_handle());
    let (drawn, lines) = page.draws("A");
    let alpha = |d: &Drawn| d.color >> 24;
    let rest_m1 = alpha(one(&drawn, "m1", &lines));
    let rest_card = alpha(all(&drawn, "card")[0]);
    eprintln!("probe2 fade: at rest m1 alpha {rest_m1:#x}, card alpha {rest_card:#x}");
    assert!(rest_m1 < 0xc0, "the row's half reaches m1 at rest");
    let a_rest = page.lines("A");
    // A hosted CSS artboard under a fade draws as one picture composited at
    // the fade (its paints stay whole), so the card's half is a line at 0.5.
    let halves = |lines: &[String]| lines.iter().filter(|l| l.contains("opacity=0.5")).count();
    let b_halves = halves(&page.lines("B"));
    eprintln!(
        "probe2 fade: A's half lines at rest: {:?}",
        a_rest.iter().filter(|l| l.contains("opacity=")).collect::<Vec<_>>()
    );
    assert!(halves(&a_rest) >= 1, "the card's picture at the row's half:\n{}", a_rest.join("\n"));
    for (element, anchor, name) in [
        (page.el("A", "m1"), page.el("B", "n1"), "m1"),
        (page.slot("A", "card"), page.slot("B", "card"), "card"),
    ] {
        Artboard::set_lifts(&a_row, vec![element.clone()], vec![]);
        Artboard::set_lifts(
            &b_row,
            vec![],
            vec![PlacedLift {
                anchor,
                row: a_row.clone(),
                element: element.clone(),
                transform: Mat2D::identity(),
            }],
        );
        let (drawn, lines) = page.draws("B");
        let without = alpha(all(&drawn, name)[0]);
        set_own_fade_only(&element, true);
        page.advance(0.0);
        let (drawn, lines2) = page.draws("B");
        let with = alpha(all(&drawn, name)[0]);
        eprintln!("probe2 fade: {name} lifted alpha without {without:#x}, with {with:#x}");
        assert_eq!(without, if name == "m1" { rest_m1 } else { rest_card }, "{}", lines.join("\n"));
        assert_eq!(with, 0xff, "{name} at its own opacity:\n{}", lines2.join("\n"));
        if name == "card" {
            assert!(halves(&lines) > b_halves, "lifted, the card's picture at the half:\n{}", lines.join("\n"));
            assert_eq!(halves(&lines2), b_halves, "with its own fade only, no half:\n{}", lines2.join("\n"));
        }
        set_own_fade_only(&element, false);
        Artboard::set_lifts(&a_row, vec![], vec![]);
        Artboard::set_lifts(&b_row, vec![], vec![]);
        page.advance(0.0);
        assert_eq!(page.lines("A"), a_rest, "{name}: A after");
    }
}

/// A text as the anchor: "note" is a paintless box holding a text, so it has
/// no box paint. The lifted card draws just before the text's first path.
#[test]
fn probe2_a_text_element_anchors_a_lift() {
    let page = Page::new(false);
    page.advance(0.1);
    let (a_row, b_row) = (page.row("A").core_handle(), page.row("B").core_handle());
    let (drawn, lines) = page.draws("B");
    let text_rest = all(&drawn, "note");
    assert!(!text_rest.is_empty(), "the note's text draws:\n{}", lines.join("\n"));
    Artboard::set_lifts(&a_row, vec![page.slot("A", "card")], vec![]);
    Artboard::set_lifts(
        &b_row,
        vec![],
        vec![PlacedLift {
            anchor: page.el("B", "note"),
            row: a_row.clone(),
            element: page.slot("A", "card"),
            transform: Mat2D::identity(),
        }],
    );
    let (drawn, lines) = page.draws("B");
    let cards = all(&drawn, "card");
    let text = all(&drawn, "note");
    let over = one(&drawn, "over", &lines).at;
    assert_eq!(cards.len(), 2, "{}", lines.join("\n"));
    // B's own card first (in its row), then over, then the lifted card,
    // then its child, then the text.
    assert!(cards[0].at < over && over < cards[1].at, "{}", lines.join("\n"));
    let lifted_child = all(&drawn, "card child")[1].at;
    assert!(lifted_child < text[0].at, "{}", lines.join("\n"));
    assert!(
        drawn.iter().filter(|d| d.at > lifted_child && d.at < text[0].at).count() == 0,
        "nothing between the lifted card and the text:\n{}",
        lines.join("\n")
    );
}

/// The resting reader during a step: a held element's own box is its
/// flight box, but `element_rest` reads its resting size from the
/// placeholder, and its resting matrix is unchanged. Today's slot reader
/// reads the flight size there. For a box and for a component placement.
#[test]
fn probe2_the_resting_reader_reads_the_placeholder_during_a_step() {
    let page = Page::new(false);
    page.advance(0.1);
    let n1 = page.el("B", "n1");
    let host = page.slot("B", "card");
    let (_, card) = page.card("B");
    let before = (element_rest(&n1).unwrap(), element_rest(&host).unwrap());
    assert_eq!(before.0.size, Vec2D::new(80.0, 100.0));
    assert_eq!(before.1.size, Vec2D::new(120.0, 80.0));
    assert!(LayoutComponent::begin_flight_hold(&n1, (60.0, 50.0)));
    assert!(LayoutComponent::begin_flight_hold(&card, (200.0, 140.0)));
    page.advance(0.0);
    assert_eq!(bounds(&n1)[2..], [60.0, 50.0]);
    assert_eq!(bounds(&card)[2..], [200.0, 140.0]);
    let during = (element_rest(&n1).unwrap(), element_rest(&host).unwrap());
    assert_eq!(during.0.size, before.0.size, "a box's rest");
    assert_eq!(during.1.size, before.1.size, "a placement's rest");
    assert_eq!(during.0.transform, before.0.transform);
    assert_eq!(during.1.transform, before.1.transform);
    let today = slot_pose(&host).unwrap().size;
    eprintln!("probe2 reader: today's slot reader during the hold reads {today:?}");
    assert_eq!(today, Vec2D::new(200.0, 140.0), "today's reader reads the live box");
    LayoutComponent::end_flight_hold(&n1);
    LayoutComponent::end_flight_hold(&card);
    page.advance(0.0);
    assert_eq!(element_rest(&n1).unwrap().size, before.0.size);
}

// Q5 ------------------------------------------------------------------------

/// Both copies lifted, each pose wrapping its lifted draw: B leaves its own
/// card out of its draw, and draws, at its card's place (the anchor keeps
/// its place though its own drawing is skipped), A's card under A's pose and
/// then its own under its pose: after n1, before over. With both poses
/// identity, B's draw is its resting draw with A's card added before its own.
#[test]
fn probe2_both_copies_lifted_with_their_poses() {
    let page = Page::new(false);
    page.advance(0.1);
    let (a_row, b_row) = (page.row("A").core_handle(), page.row("B").core_handle());
    let (a_card, b_card) = (page.slot("A", "card"), page.slot("B", "card"));
    let b_rest = page.lines("B");
    Artboard::set_lifts(&a_row, vec![a_card.clone()], vec![]);
    for (pose_a, pose_b, want_a, want_b) in [
        (
            Mat2D::identity(),
            Mat2D::identity(),
            r4(110.0, 0.0, 270.0, 100.0),
            r4(90.0, 0.0, 210.0, 80.0),
        ),
        (
            Mat2D::from_translate(-20.0, 10.0),
            Mat2D::from_translate(30.0, 50.0),
            r4(90.0, 10.0, 250.0, 110.0),
            r4(120.0, 50.0, 240.0, 130.0),
        ),
    ] {
        Artboard::set_lifts(
            &b_row,
            vec![b_card.clone()],
            vec![
                PlacedLift {
                    anchor: b_card.clone(),
                    row: a_row.clone(),
                    element: a_card.clone(),
                    transform: pose_a,
                },
                PlacedLift {
                    anchor: b_card.clone(),
                    row: b_row.clone(),
                    element: b_card.clone(),
                    transform: pose_b,
                },
            ],
        );
        let (drawn, lines) = page.draws("B");
        let cards = all(&drawn, "card");
        assert_eq!(cards.len(), 2, "{}", lines.join("\n"));
        assert!(near(cards[0].rect, want_a), "A's card {:?}", cards[0]);
        assert!(near(cards[1].rect, want_b), "B's card {:?}", cards[1]);
        let n1 = one(&drawn, "n1", &lines).at;
        let over = one(&drawn, "over", &lines).at;
        assert!(n1 < cards[0].at && cards[1].at < over, "{}", lines.join("\n"));
        assert_eq!(drawpaths(&lines), drawpaths(&b_rest) + 2);
    }
    Artboard::set_lifts(&a_row, vec![], vec![]);
    Artboard::set_lifts(&b_row, vec![], vec![]);
    assert_eq!(page.lines("B"), b_rest);
}

// Cost ----------------------------------------------------------------------

/// Per frame, debug build, no caches: the component held at a changing size
/// (its Screen solved with the hold) next to today's unit path (the host's
/// record at that size, the Screen solved), and the card alone as a root at
/// that size. Printed, not asserted.
#[test]
fn probe2_cost_per_frame_component() {
    use std::time::Instant;
    const N: usize = 200;
    let size = |i: usize| lerp((120.0, 80.0), (160.0, 100.0), (i % 20) as f32 / 20.0);
    let held = Page::new(false);
    held.advance(0.1);
    let (_, card) = held.card("B");
    assert!(LayoutComponent::begin_flight_hold(&card, size(0)));
    let start = Instant::now();
    for i in 0..N {
        LayoutComponent::set_flight_hold_size(&card, size(i + 1));
        held.advance(0.0);
    }
    let hold = start.elapsed().as_secs_f64() * 1e6 / N as f64;
    let today = Page::new(false);
    today.advance(0.1);
    let slot = today.slot("B", "card");
    let start = Instant::now();
    for i in 0..N {
        size_host(&slot, size(i + 1));
        today.advance(0.0);
    }
    let host = start.elapsed().as_secs_f64() * 1e6 / N as f64;
    let rest = Page::new(false);
    rest.advance(0.1);
    let start = Instant::now();
    for _ in 0..N {
        rest.advance(0.0);
    }
    let still = start.elapsed().as_secs_f64() * 1e6 / N as f64;
    let alone = held.file.with_file(|f| f.artboard_named("card")).unwrap();
    alone.advance_default(0.0);
    let start = Instant::now();
    for i in 0..N {
        let (w, h) = size(i + 1);
        alone.set_size(w, h);
        alone.advance_default(0.0);
    }
    let unit = start.elapsed().as_secs_f64() * 1e6 / N as f64;
    let (a_row, b_row) = (held.row("A").core_handle(), held.row("B").core_handle());
    LayoutComponent::end_flight_hold(&card);
    held.advance(0.0);
    let draw = |page: &Page| {
        let start = Instant::now();
        for _ in 0..N {
            page.factory.borrow_mut().clear();
            let mut renderer = page.factory.borrow().make_renderer();
            page.row("A").draw(&mut renderer);
            page.row("B").draw(&mut renderer);
        }
        start.elapsed().as_secs_f64() * 1e6 / N as f64
    };
    let plain = draw(&held);
    Artboard::set_lifts(&a_row, vec![held.slot("A", "card")], vec![]);
    Artboard::set_lifts(
        &b_row,
        vec![],
        vec![PlacedLift {
            anchor: held.slot("B", "card"),
            row: a_row.clone(),
            element: held.slot("A", "card"),
            transform: Mat2D::identity(),
        }],
    );
    let lifted = draw(&held);
    eprintln!(
        "probe2 cost per frame (us): rest {still:.1}, held component {hold:.1}, today's host record {host:.1}, card alone as a root {unit:.1}; draw both Screens plain {plain:.1}, lifted {lifted:.1}"
    );
}
