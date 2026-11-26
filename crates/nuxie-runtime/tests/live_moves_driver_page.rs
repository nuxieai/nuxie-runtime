//! Named steps between two rows of a driver page (docs/live-moves.md, Steps
//! on a driver page). The page's keyed list "rows" is marked `isPageRows`,
//! a driver mounts its rows by key from Screen A or Screen B, and a step
//! names its two rows by key: `step_begin_between` and `step_end_between`.
//! The step's from and to artboards name its kind, so the same component
//! twice is two rows, and the step reads its flight records from the upper
//! row alone, the one later in the list's order.
//!
//! Oracle: the same flights on a `screens` page (tests/live_moves.rs), whose
//! records the compiler writes on the upper Screen; each row's own instance,
//! which holds its own copy of every element and record; and the list's
//! order, in which a later row draws above an earlier one.
use std::cell::RefCell;
use std::rc::Rc;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::live_move::{LiveMoves, records::LiveFlight, screens};
use nuxie_runtime::live_move_fixture::{
    self as fixture, DRIVER_A as A, DRIVER_B as B, DRIVER_ROWS, DriverPage,
};
use nuxie_runtime::page_player::{
    self, DriverFault, DriverRound, ErrorKind, HostDriver, PagePlayer, Playback, SceneObject,
    SceneWriter, Step,
};
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList, generated::core_registry::CoreRegistry,
    nested_artboard::NestedArtboard,
};
use nuxie_runtime::{
    ArtboardInstance, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeOwnedViewModelHandle, hosted_scene, source,
};

/// The fills of the units: A's version, and B's.
const RED: u32 = 0xff0000;
const BLUE: u32 = 0x0000ff;
/// A flight record's progress and fade.
const PROGRESS: i32 = 62033;
const FADE: i32 = 62036;
/// A layout component's opacity.
const OPACITY: i32 = 18;

/// Writes a driver makes in round 0 of the next step.
type Write = Box<dyn FnOnce(&mut SceneWriter<'_>)>;

/// A driver that makes the queued writes in round 0 of each step.
struct Writes(Rc<RefCell<Vec<Write>>>);

impl HostDriver for Writes {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault> {
        if round.index == 0 {
            for write in self.0.take() {
                write(scene);
            }
        }
        Ok(())
    }
}

/// The page's list as a driver names it.
fn rows_list() -> SceneObject {
    SceneObject::root(DRIVER_ROWS)
}

fn val(h: &CoreHandle, key: i32) -> f32 {
    CoreRegistry::get_double_handle(h, key).unwrap()
}

fn set(h: &CoreHandle, key: i32, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, key, v));
}

fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.002, "expected {b}, got {a}");
}

/// The instance a placement holds.
fn holds(slot: &CoreHandle) -> CoreHandle {
    slot.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
        .expect("the placement holds an instance")
        .core_handle()
}

/// The keyed box a placement is in: the element that flies.
fn box_of(slot: &CoreHandle) -> CoreHandle {
    slot.with(|o| o.component_parent_handle())
        .flatten()
        .unwrap()
}

/// A row's flight records.
fn records_of(row: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    row.with_artboard(|b| b.find_all_handles::<LiveFlight>())
}

fn key_of(record: &CoreHandle) -> u32 {
    record
        .with_downcast::<LiveFlight, _>(|r| r.values.key)
        .unwrap()
}

fn refused<T: std::fmt::Debug>(result: Result<T, page_player::Error>) -> ErrorKind {
    result.expect_err("refused").kind
}

/// Left, top, width and height.
fn bounds_of(points: &[source::math::vec2d::Vec2D]) -> [f32; 4] {
    let low = |of: fn(&source::math::vec2d::Vec2D) -> f32| {
        points.iter().map(of).fold(f32::INFINITY, f32::min)
    };
    let high = |of: fn(&source::math::vec2d::Vec2D) -> f32| {
        points.iter().map(of).fold(f32::NEG_INFINITY, f32::max)
    };
    let (left, top) = (low(|p| p.x), low(|p| p.y));
    [left, top, high(|p| p.x) - left, high(|p| p.y) - top]
}

fn import(bytes: &[u8]) -> (RuntimeFileHandle, PersistentFactory<RecordingFactory>) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("the fixture imports");
    (file, factory)
}

fn imports(bytes: &[u8]) -> bool {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .is_some()
}

/// A driver page played by a page player, with a driver that mounts,
/// removes, moves and writes rows between steps.
struct DriverScene {
    _file: RuntimeFileHandle,
    factory: PersistentFactory<RecordingFactory>,
    board: ArtboardInstance,
    playback: Playback,
    player: PagePlayer,
    writes: Rc<RefCell<Vec<Write>>>,
    moves: LiveMoves,
}

impl DriverScene {
    /// `kind`'s driver page with a row mounted from the artboard beside
    /// each key, in order, in its first step.
    fn new(kind: DriverPage, rows: &[(&'static str, u32)]) -> Self {
        let (file, factory) = import(&fixture::driver_page_fixture(kind));
        let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let machine = board
            .default_state_machine_instance()
            .expect("the page's machine");
        let writes = Rc::<RefCell<Vec<Write>>>::default();
        let player = PagePlayer::default();
        player
            .set_driver(Some(Box::new(Writes(writes.clone()))))
            .expect("no step is running");
        let mut scene = Self {
            _file: file,
            factory,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            player,
            writes,
            moves: LiveMoves::default(),
        };
        scene.mount(rows);
        assert!(hosted_scene::enabled(&scene.root()), "a driver page");
        scene
    }

    fn root(&self) -> RuntimeArtboardInstanceHandle {
        self.board.native_handle()
    }

    fn write(&self, write: impl FnOnce(&mut SceneWriter<'_>) + 'static) {
        self.writes.borrow_mut().push(Box::new(write));
    }

    fn step(&mut self, elapsed_seconds: f32) {
        self.player
            .step(
                &mut self.board,
                &mut self.playback,
                Step {
                    elapsed_seconds,
                    ..Step::default()
                },
                &[],
                None,
            )
            .expect("the step");
    }

    /// Mount a row for each key, from the artboard beside it, in one step.
    fn mount(&mut self, rows: &[(&'static str, u32)]) {
        let rows = rows.to_vec();
        self.write(move |scene| {
            for (key, artboard) in rows {
                assert_eq!(
                    scene.mount_row_from(&rows_list(), key.as_bytes(), artboard),
                    Ok(true)
                );
            }
        });
        self.step(0.0);
    }

    /// Remove the row mounted by `key`, in the next step.
    fn remove(&mut self, key: &'static str) {
        self.write(move |scene| {
            assert_eq!(scene.remove_row(&rows_list(), key.as_bytes()), Ok(true));
        });
        self.step(0.0);
    }

    fn list(&self) -> CoreHandle {
        self.root()
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("rows"))
            .expect("the page's list")
    }

    /// The keys of the list's rows, in order.
    fn keys(&self) -> Vec<String> {
        self.list()
            .with_downcast::<ArtboardComponentList, _>(|list| {
                list.row_keys()
                    .into_iter()
                    .map(|key| String::from_utf8_lossy(key).into_owned())
                    .collect()
            })
            .unwrap()
    }

    /// The row mounted by `key`.
    fn row(&self, key: &str) -> RuntimeArtboardInstanceHandle {
        self.list()
            .with_downcast::<ArtboardComponentList, _>(|list| {
                let place = list
                    .row_keys()
                    .iter()
                    .position(|row| *row == key.as_bytes())?;
                list.artboard_instance(place as i32)
            })
            .flatten()
            .unwrap_or_else(|| panic!("no row {key}"))
    }

    /// The unit's placement on the row mounted by `key`.
    fn slot(&self, key: &str) -> CoreHandle {
        self.row(key)
            .with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0]
            .clone()
    }

    fn begin(
        &mut self,
        outgoing: &str,
        incoming: &str,
        name: &str,
    ) -> Result<(), page_player::Error> {
        self.player.step_begin_between(
            &mut self.board,
            outgoing.as_bytes(),
            incoming.as_bytes(),
            name,
            &[],
            None,
        )
    }

    fn end(
        &mut self,
        outgoing: &str,
        incoming: &str,
        name: &str,
    ) -> Result<(), page_player::Error> {
        self.player.step_end_between(
            &mut self.board,
            outgoing.as_bytes(),
            incoming.as_bytes(),
            name,
            &[],
            None,
        )
    }

    fn sleep(&mut self, key: &str, collapsed: bool) {
        self.player
            .set_keyed_row_collapsed(
                &mut self.board,
                "rows",
                key.as_bytes(),
                collapsed,
                &[],
                None,
            )
            .expect("the row sleeps or wakes");
    }

    /// Each flying copy: its element, its row and its flight record.
    fn lifted(&self) -> Vec<(CoreHandle, CoreHandle, CoreHandle)> {
        self.moves.lifted_records(&self.root())
    }

    fn poisoned(&self) -> bool {
        page_player::is_poisoned(&self.root())
    }

    /// What the page draws of the unit filled with `rgb`, in draw order:
    /// its fill's box on the page, and its opacity.
    fn drawn(&self, rgb: u32) -> Vec<([f32; 4], f32)> {
        self.factory.borrow_mut().clear();
        let mut renderer = self.factory.borrow().make_renderer();
        self.root().draw(&mut renderer);
        let stream = self.factory.borrow().stream();
        nuxie_render_stream::frame_draws(&nuxie_render_stream::stream_ops(&stream).unwrap(), &[])
            .iter()
            .filter(|draw| draw.rgb() == Some(rgb))
            .map(|draw| {
                let points = draw
                    .points
                    .iter()
                    .map(|p| source::math::vec2d::Vec2D::new(p.x, p.y))
                    .collect::<Vec<_>>();
                (bounds_of(&points), draw.opacity())
            })
            .collect()
    }

    fn drawn_once(&self, rgb: u32) -> ([f32; 4], f32) {
        let draws = self.drawn(rgb);
        assert_eq!(draws.len(), 1, "one draw of {rgb:06x}");
        draws[0]
    }

    /// The draw lines of the page, in order.
    fn draw_lines(&self) -> Vec<String> {
        self.factory.borrow_mut().clear();
        let mut renderer = self.factory.borrow().make_renderer();
        self.root().draw(&mut renderer);
        let stream = self.factory.borrow().stream();
        stream
            .lines()
            .filter(|line| line.starts_with("drawPath "))
            .map(str::to_owned)
            .collect()
    }
}

/// Proof 1, from `two_artboard_versions_keep_identity_and_draw_bs_copy_on_top`.
/// Forward t1 from row "a" to row "b", then back with "b" still the upper
/// row: each copy's record is the one its step names, on the upper row, each
/// placement keeps its instance, and B's copy draws on top both ways.
#[test]
fn two_artboard_versions_keep_identity_and_draw_bs_copy_on_top() {
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("a", A), ("b", B)]);
    let (sa, sb) = (s.slot("a"), s.slot("b"));
    let (a, b) = (holds(&sa), holds(&sb));
    for (outgoing, incoming, step, keys) in
        [("a", "b", "t1", [11, 12]), ("b", "a", "back", [13, 14])]
    {
        s.begin(outgoing, incoming, step).expect("the step begins");
        let flights = s.lifted();
        assert_eq!(flights.len(), 2);
        let upper = records_of(&s.row("b"));
        // A's copy is the leaving one of t1, and the arriving one of back.
        for (element, key) in [(box_of(&sa), keys[0]), (box_of(&sb), keys[1])] {
            let record = &flights.iter().find(|f| f.0 == element).unwrap().2;
            assert_eq!(key_of(record), key);
            assert!(upper.contains(record), "read from the upper row");
            set(record, PROGRESS, 0.5);
        }
        assert_eq!(holds(&sa), a);
        assert_eq!(holds(&sb), b);
        s.step(0.0);
        let draws = s.draw_lines();
        assert_eq!(draws.len(), 2, "exactly the two versions draw");
        for (draw, color) in draws.iter().zip(["color=0xffff0000", "color=0xff0000ff"]) {
            assert!(draw.contains(color), "{draw}");
        }
        s.end(outgoing, incoming, step).expect("the step lands");
        assert!(s.lifted().is_empty());
        assert_eq!(holds(&sa), a);
        assert_eq!(holds(&sb), b);
    }
    assert!(!s.poisoned());
}

/// Proof 1, from `two_copies_of_one_key_each_play_their_own_half`: B's
/// entrance, played by the upper row's machine, keys t1's crossfade, the
/// leaving copy its Exit and the arriving copy its Entrance.
#[test]
fn two_copies_of_one_key_each_play_their_own_half() {
    let mut s = DriverScene::new(DriverPage::SameKeyCopies, &[("a", A), ("b", B)]);
    let (sa, sb) = (s.slot("a"), s.slot("b"));
    let (a, b) = (holds(&sa), holds(&sb));
    s.begin("a", "b", "t1").expect("the step begins");
    s.step(0.25);
    let flights = s.lifted();
    assert_eq!(flights.len(), 2);
    for (element, _, record) in &flights {
        let leaving = element == &box_of(&sa);
        assert!(leaving || element == &box_of(&sb));
        close(val(record, FADE), if leaving { 0.75 } else { 0.25 });
        close(val(record, PROGRESS), 0.25);
    }
    s.step(0.75);
    s.end("a", "b", "t1").expect("the step lands");
    assert_eq!(holds(&sa), a);
    assert_eq!(holds(&sb), b);
}

/// Proof 1, from `two_version_fade_multiplies_blended_opacity_at_each_progress`,
/// at one progress: an opacity each element has itself is its copy's own,
/// times the copy's fade, and both copies are where the flight has them.
#[test]
fn two_version_fade_multiplies_own_opacity_at_one_progress() {
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("a", A), ("b", B)]);
    let (rest_a, _) = s.drawn_once(RED);
    let (rest_b, _) = s.drawn_once(BLUE);
    let (sa, sb) = (s.slot("a"), s.slot("b"));
    set(&box_of(&sa), OPACITY, 0.4);
    set(&box_of(&sb), OPACITY, 0.8);
    s.step(0.0);
    s.begin("a", "b", "t1").expect("the step begins");
    let t = 0.25;
    let fades = [((1.0 - t) * 2.0_f32).min(1.0), (t * 2.0_f32).min(1.0)];
    for (element, _, record) in s.lifted() {
        let arriving = element == box_of(&sb);
        set(&record, PROGRESS, t);
        set(&record, FADE, fades[usize::from(arriving)]);
    }
    s.step(0.0);
    for (rgb, own, fade) in [(RED, 0.4, fades[0]), (BLUE, 0.8, fades[1])] {
        let (drawn, opacity) = s.drawn_once(rgb);
        close(opacity, own * fade);
        close(drawn[0], rest_a[0] + (rest_b[0] - rest_a[0]) * t);
    }
    s.end("a", "b", "t1").expect("the step lands");
}

/// Proof 1, from `native_keyframes_drive_a_flight_records_progress`: B's
/// entrance, which the upper row's machine plays, drives the step's
/// records on that row, from the first frame begin applies.
#[test]
fn native_keyframes_drive_the_upper_rows_flight_records() {
    let mut s = DriverScene::new(DriverPage::Keyed, &[("a", A), ("b", B)]);
    s.begin("a", "b", "t1").expect("the step begins");
    let upper = records_of(&s.row("b"));
    let flights = s.lifted();
    assert_eq!(flights.len(), 2);
    for (_, _, record) in &flights {
        assert!(upper.contains(record));
        close(val(record, PROGRESS), 0.0);
    }
    s.step(0.5);
    for (_, _, record) in &flights {
        close(val(record, PROGRESS), 0.5);
    }
    s.end("a", "b", "t1").expect("the step lands");
}

/// Proof 2 and 4. Two rows of one component, "a1" and "a2", with a third
/// row "b" asleep between them. Step `same` flies both copies, each in its
/// own row: every record is the upper row's, a2's, the host plays the
/// flight on a2 alone, so a1's records are untouched, and a2's copy draws
/// on top. "b" is not drawn and not read. Landing leaves every placement's
/// instance and every row in place. The host sleeps a1, then wakes it and
/// moves it just below a2, and `same.reverse` from a2 back to a1 reads its
/// records from a2, the outgoing row and now the upper one; a1's copy is
/// the one it was left with.
#[test]
fn two_rows_of_one_component_fly_each_copy_in_its_own_row() {
    let mut s = DriverScene::new(DriverPage::SameComponent, &[("a1", A), ("b", B), ("a2", A)]);
    s.sleep("b", true);
    s.step(0.0);
    let (r1, r2) = (s.row("a1"), s.row("a2"));
    let (s1, s2) = (s.slot("a1"), s.slot("a2"));
    let (h1, h2) = (holds(&s1), holds(&s2));
    let lower = records_of(&r1);
    let before = lower
        .iter()
        .map(|record| (val(record, PROGRESS), val(record, FADE)))
        .collect::<Vec<_>>();
    let b_records = records_of(&s.row("b"))
        .iter()
        .map(|record| val(record, PROGRESS))
        .collect::<Vec<_>>();

    s.begin("a1", "a2", "same").expect("the step begins");
    let flights = s.lifted();
    assert_eq!(flights.len(), 2, "both copies fly");
    let upper = records_of(&r2);
    for (element, row, record) in &flights {
        let (own_row, key) = if *element == box_of(&s1) {
            (&r1, 21)
        } else {
            assert_eq!(*element, box_of(&s2));
            (&r2, 22)
        };
        assert_eq!(
            *row,
            own_row.core_handle(),
            "each copy flies in its own row"
        );
        assert!(upper.contains(record), "every record is the upper row's");
        assert_eq!(key_of(record), key);
    }
    // The host plays the flight on the upper row's half.
    s.write(|scene| {
        assert_eq!(
            scene.set_row_input_bool(&rows_list(), b"a2", "fly", true),
            Ok(true)
        );
    });
    s.step(0.0);
    s.step(0.25);
    for (element, _, record) in &flights {
        let leaving = *element == box_of(&s1);
        close(val(record, PROGRESS), 0.25);
        close(val(record, FADE), if leaving { 0.75 } else { 0.25 });
    }
    let after = lower
        .iter()
        .map(|record| (val(record, PROGRESS), val(record, FADE)))
        .collect::<Vec<_>>();
    assert_eq!(after, before, "the lower row's records are untouched");
    // Both copies draw where the flight has them, a2's on top; "b" draws
    // nothing.
    let draws = s.drawn(RED);
    assert_eq!(draws.len(), 2);
    close(draws[0].1, 0.75);
    close(draws[1].1, 0.25);
    assert!(s.drawn(BLUE).is_empty(), "the sleeping row is not drawn");
    s.step(0.75);
    s.end("a1", "a2", "same").expect("the step lands");
    assert!(s.lifted().is_empty());
    assert_eq!(holds(&s1), h1);
    assert_eq!(holds(&s2), h2);
    assert_eq!(s.row("a1").core_handle(), r1.core_handle());
    assert_eq!(s.row("a2").core_handle(), r2.core_handle());
    let b_after = records_of(&s.row("b"))
        .iter()
        .map(|record| val(record, PROGRESS))
        .collect::<Vec<_>>();
    assert_eq!(b_after, b_records, "the sleeping row is not read");

    // Back: a1, slept after landing, wakes just below a2.
    s.sleep("a1", true);
    s.step(0.0);
    s.sleep("a1", false);
    s.write(|scene| {
        assert_eq!(scene.move_row(&rows_list(), b"a1", 1), Ok(true));
    });
    s.step(0.0);
    assert_eq!(s.keys(), ["b", "a1", "a2"]);
    assert_eq!(holds(&s1), h1, "a1 wakes as it was left");
    s.begin("a2", "a1", "same.reverse")
        .expect("the reverse begins");
    let flights = s.lifted();
    assert_eq!(flights.len(), 2);
    for (element, row, record) in &flights {
        let (own_row, key) = if *element == box_of(&s2) {
            (&r2, 24)
        } else {
            assert_eq!(*element, box_of(&s1));
            (&r1, 23)
        };
        assert_eq!(*row, own_row.core_handle());
        assert!(upper.contains(record), "read from a2, the upper row");
        assert_eq!(key_of(record), key);
    }
    s.end("a2", "a1", "same.reverse")
        .expect("the reverse lands");
    assert_eq!(holds(&s1), h1);
    assert_eq!(holds(&s2), h2);
    assert!(!s.poisoned());
}

/// Proof 3. Removing a step's row mid-step, the incoming one and then the
/// outgoing one, cancels the step before the next frame: nothing flies,
/// the row left draws its unit at rest, the end is refused, and a nameless
/// step can follow. Removing an asleep third row does nothing to the step.
#[test]
fn removing_either_row_mid_step_cancels_before_the_next_frame() {
    for (gone, left, rgb) in [("b", "a", RED), ("a", "b", BLUE)] {
        let mut s = DriverScene::new(DriverPage::TwoVersions, &[("a", A), ("b", B), ("c", A)]);
        s.sleep("c", true);
        s.step(0.0);
        let (rest, _) = s.drawn_once(rgb);
        s.begin("a", "b", "t1").expect("the step begins");
        for (_, _, record) in s.lifted() {
            set(&record, PROGRESS, 0.5);
        }
        s.step(0.0);
        s.remove("c");
        assert_eq!(s.lifted().len(), 2, "an asleep row left; the step runs on");
        s.remove(gone);
        assert!(s.lifted().is_empty(), "the step cancels in that step");
        let (drawn, opacity) = s.drawn_once(rgb);
        assert_eq!(drawn, rest, "{left} settles at rest");
        close(opacity, 1.0);
        assert_eq!(refused(s.end("a", "b", "t1")), ErrorKind::InvalidArgument);
        assert!(!s.poisoned());
        s.player
            .step_begin_row(&mut s.board, left.as_bytes(), &[], None)
            .expect("a nameless step follows");
        s.player
            .step_end_row(&mut s.board, left.as_bytes(), &[], None)
            .expect("and lands");
    }
}

/// Proof 3. A key removed and mounted again in one round is a new row, so a
/// step holding the old one cancels, its end is refused, and the step can
/// begin again with the new row.
#[test]
fn a_key_removed_and_mounted_again_in_one_round_cancels_the_step() {
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("a", A), ("b", B)]);
    let old = s.row("b").core_handle();
    s.begin("a", "b", "t1").expect("the step begins");
    s.write(|scene| {
        assert_eq!(scene.remove_row(&rows_list(), b"b"), Ok(true));
        assert_eq!(scene.mount_row_from(&rows_list(), b"b", B), Ok(true));
    });
    s.step(0.0);
    assert_ne!(s.row("b").core_handle(), old, "a new row");
    assert!(s.lifted().is_empty(), "the step cancels");
    assert_eq!(refused(s.end("a", "b", "t1")), ErrorKind::InvalidArgument);
    assert!(!s.poisoned());
    s.begin("a", "b", "t1").expect("the new row can arrive");
    assert_eq!(s.lifted().len(), 2);
    s.end("a", "b", "t1").expect("and land");
}

/// Proof 5. Each refusal changes nothing and marks nothing: nothing flies,
/// the occurrence stays usable, and the step can still begin after them.
#[test]
fn refusals_mark_nothing() {
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("a", A), ("b", B), ("c", A)]);
    s.sleep("c", true);
    s.step(0.0);
    let (sa, sb) = (s.slot("a"), s.slot("b"));
    let (a, b) = (holds(&sa), holds(&sb));
    let check = |s: &DriverScene, why: &str| {
        assert!(s.lifted().is_empty(), "{why}");
        assert!(!s.poisoned(), "{why}");
        assert_eq!(holds(&sa), a, "{why}");
        assert_eq!(holds(&sb), b, "{why}");
    };
    for (outgoing, incoming, name, why) in [
        ("a", "b", "nope", "an unknown name"),
        ("b", "a", "t1", "a kind mismatch"),
        ("a", "a", "t1", "the same key twice"),
        ("a", "gone", "t1", "an unknown key"),
        ("gone", "b", "t1", "an unknown outgoing key"),
        ("c", "b", "t1", "an asleep row"),
    ] {
        assert_eq!(
            refused(s.begin(outgoing, incoming, name)),
            ErrorKind::InvalidArgument,
            "{why}"
        );
        check(&s, why);
    }
    // The name form of a step is refused on a driver page.
    let by_name = s
        .player
        .step_begin(&mut s.board, "B", Some("t1"), &[], None);
    assert_eq!(refused(by_name), ErrorKind::InvalidArgument);
    check(&s, "a step named by Screen");

    s.begin("a", "b", "t1").expect("the step begins");
    assert_eq!(
        refused(s.begin("b", "a", "back")),
        ErrorKind::InvalidArgument,
        "a step is running"
    );
    for (outgoing, incoming, name) in [("a", "b", "back"), ("b", "a", "t1"), ("a", "c", "t1")] {
        assert_eq!(
            refused(s.end(outgoing, incoming, name)),
            ErrorKind::InvalidArgument,
            "an end that names another step"
        );
    }
    assert_eq!(s.lifted().len(), 2, "the step runs on");
    s.end("a", "b", "t1").expect("the step lands");
    assert_eq!(
        refused(s.end("a", "b", "t1")),
        ErrorKind::InvalidArgument,
        "no step is running"
    );
    check(&s, "after the step");

    // Records only on the lower row: with "b" first, "a" is the upper row,
    // and A holds none of t1's records.
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("b", B), ("a", A)]);
    assert_eq!(refused(s.begin("a", "b", "t1")), ErrorKind::InvalidArgument);
    assert!(s.lifted().is_empty());
    assert!(!s.poisoned());

    // A unit that carries an instance between rows.
    let mut s = DriverScene::new(DriverPage::Transfers, &[("a", A), ("b", B)]);
    let (sa, sb) = (s.slot("a"), s.slot("b"));
    let (a, b) = (holds(&sa), holds(&sb));
    assert_eq!(refused(s.begin("a", "b", "t1")), ErrorKind::InvalidArgument);
    assert!(s.lifted().is_empty());
    assert!(!s.poisoned());
    assert_eq!((holds(&sa), holds(&sb)), (a, b));

    // A `screens` page has no rows by key.
    let (file, _factory) = import(&fixture::two_version_fixture());
    let mut board = ArtboardInstance::from_native(file, 0).unwrap();
    let player = PagePlayer::default();
    let begin = player.step_begin_between(&mut board, b"a", b"b", "t1", &[], None);
    assert_eq!(refused(begin), ErrorKind::InvalidArgument);
}

/// Proof 5, with a layout change pending. With "b" first, "a" is the upper
/// row and holds none of t1's records. The root is resized, so a flush
/// would lay the page out again: the step is refused before it, the page
/// is not laid out by the refusal and stays usable. Moved below "a", "b"
/// is the upper row, and the same step begins and lands.
#[test]
fn a_step_refused_with_layout_pending_leaves_the_page_usable() {
    let mut s = DriverScene::new(DriverPage::TwoVersions, &[("b", B), ("a", A)]);
    let root = s.root();
    root.set_size(321.0, 654.0);
    let serial = root.with_artboard(|b| b.change_serial());
    assert_eq!(refused(s.begin("a", "b", "t1")), ErrorKind::InvalidArgument);
    assert_eq!(
        root.with_artboard(|b| b.change_serial()),
        serial,
        "the refusal flushes nothing"
    );
    assert!(s.lifted().is_empty());
    assert!(!s.poisoned());

    s.write(|scene| {
        assert_eq!(scene.move_row(&rows_list(), b"b", 1), Ok(true));
    });
    s.step(0.0);
    assert_eq!(s.keys(), ["a", "b"]);
    s.begin("a", "b", "t1").expect("the step begins");
    let upper = records_of(&s.row("b"));
    let flights = s.lifted();
    assert_eq!(flights.len(), 2);
    assert!(flights.iter().all(|(_, _, record)| upper.contains(record)));
    s.step(0.0);
    s.end("a", "b", "t1").expect("the step lands");
    assert!(s.lifted().is_empty());
    assert!(!s.poisoned());
}

/// Proof 6. A component artboard of a driver page is checked as a Screen
/// at import: a duplicate flight key refuses the file, and so does an
/// invalid unit. A driver page with steps and no step running frames as a
/// record-free one does: its frames walk the same.
#[test]
fn a_driver_pages_steps_and_screens_are_checked_at_import() {
    assert!(imports(&fixture::driver_page_fixture(
        DriverPage::TwoVersions
    )));
    assert!(!imports(&fixture::driver_page_fixture(
        DriverPage::DuplicateFlightKey
    )));
    assert!(!imports(&fixture::driver_page_fixture(
        DriverPage::InvalidUnit
    )));

    let walks = |kind| {
        let mut s = DriverScene::new(kind, &[("a", A), ("b", B)]);
        s.step(0.0);
        hosted_scene::measure_walks(true);
        for _ in 0..8 {
            s.step(1.0 / 60.0);
        }
        hosted_scene::measure_walks(false)
    };
    assert_eq!(
        walks(DriverPage::TwoVersions),
        walks(DriverPage::RecordFree)
    );
}

/// Proof 8. A `screens` page still refuses a Screen held twice: its rows
/// are named by artboard.
#[test]
fn a_screens_page_still_refuses_a_screen_held_twice() {
    let (file, _factory) = import(&fixture::two_version_fixture());
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let vm = RuntimeOwnedViewModelHandle::from_native(file.clone(), model.clone()).unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    assert_eq!(screens(&root).unwrap().len(), 2);
    let a = vm.list_item_by_property_name_path("screens", 0).unwrap();
    assert!(vm.insert_list_item_by_property_name_path("screens", 2, &a));
    machine.advance_and_apply(0.0);
    let Err(error) = screens(&root) else {
        panic!("a Screen held twice is refused");
    };
    assert!(error.to_string().contains("appears twice"), "{error}");
    assert!(
        LiveMoves::default()
            .step_begin(&root, "B", Some("t1"))
            .is_err()
    );
}
