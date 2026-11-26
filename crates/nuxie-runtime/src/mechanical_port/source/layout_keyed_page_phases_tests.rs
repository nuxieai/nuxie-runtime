//! A driver writing phases into the machines of a driver page's rows, and a
//! nameless step whose arriving end is a row named by its key. The plans
//! screen's "Cards" list is the page's rows (`Cards::PagePhases`), and each
//! card's machine has number inputs "phase" and "step" and a bool input
//! "reduceMotion": its "enter" layer plays "enter" (the dot moves right) for
//! the entering phase at step 7, its "exit" layer plays "exit" (the dot moves
//! down) for the exiting phase, each fires its event on its last frame, and
//! each goes back to idle when the phase or the step changes.
//!
//! Oracle: the card machine's file: which state each layer is in for the
//! inputs written, the animations' length (30 frames), and that a one-shot
//! animation crosses its last frame once; the row each write named.
use super::*;
use crate::hosted_scene;
use crate::keyed_rows_fixture::{PHASE_ENTERING, PHASE_EXITING, STEP};
use crate::page_player::{Error, ErrorKind};
use crate::source::animation::state_machine_instance::RuntimeStateMachineInstanceHandle;

const CARD: u32 = fixture::CARD_ARTBOARD;
const BANNER: u32 = fixture::BANNER_ARTBOARD;

/// Writes a driver makes in round 0 of the next step.
type Write = Box<dyn FnOnce(&mut SceneWriter<'_>)>;

/// One report: its name, its path and the keys of its rows.
type Heard = (String, Vec<PathStep>, Vec<String>);

/// A driver that makes queued writes in the next step's round 0, and hears
/// every report of every round.
struct Host {
    writes: Rc<RefCell<Vec<Write>>>,
    heard: Rc<RefCell<Vec<Heard>>>,
}

impl Host {
    fn on(played: &Played) -> Self {
        let host = Self {
            writes: Rc::default(),
            heard: Rc::default(),
        };
        let (writes, heard) = (host.writes.clone(), host.heard.clone());
        played.drive(move |round, scene| {
            if round.index == 0 {
                for write in writes.take() {
                    write(scene);
                }
            }
            heard.borrow_mut().extend(round.reports.iter().map(report));
        });
        host
    }

    fn write(&self, write: impl FnOnce(&mut SceneWriter<'_>) + 'static) {
        self.writes.borrow_mut().push(Box::new(write));
    }

    fn take(&self) -> Vec<Heard> {
        self.heard.take()
    }
}

fn report(event: &OwnedPlayerEvent) -> Heard {
    let context = event.context.as_ref().expect("a context");
    (
        String::from_utf8_lossy(event.name.as_deref().unwrap_or_default()).into_owned(),
        context.path().to_vec(),
        context
            .rows()
            .iter()
            .map(|row| String::from_utf8_lossy(&row.key).into_owned())
            .collect(),
    )
}

/// `cards` played with a row mounted from the artboard beside each key,
/// settled.
fn page_with(cards: Cards, rows: &[(&'static str, u32)]) -> Played {
    let mut played = played(cards);
    mount_from(&mut played, rows);
    played.frames_until_settled();
    played
}

/// The machine the list plays on the row mounted by `key`.
fn machine(played: &Played, key: &str) -> RuntimeStateMachineInstanceHandle {
    list(played)
        .with_downcast::<ArtboardComponentList, _>(|list| list.keyed_row_machine(key.as_bytes()))
        .expect("the list")
        .expect("a mounted row")
        .expect("a card machine")
}

/// How many of the row's layers play an animation: 0 when both are idle.
fn playing(played: &Played, key: &str) -> usize {
    machine(played, key).with_instance_mut(|machine| machine.current_animation_count())
}

fn phase(played: &Played, key: &str) -> f32 {
    machine(played, key)
        .with_instance(|machine| machine.get_number("phase").expect("a phase input").value())
}

fn dot_y(played: &Played, key: &str) -> f32 {
    CoreRegistry::get_double_handle(
        &row_object(played, key, card::DOT),
        i32::from(NodeBase::Y_PROPERTY_KEY),
    )
    .expect("the dot's y")
}

/// Write the phase and the step into the row mounted by `key`.
fn write_phase(host: &Host, key: &'static str, phase: f32, step: f32) {
    host.write(move |scene| {
        scene
            .set_row_input_number(&cards(), key.as_bytes(), "phase", phase)
            .expect("the phase is written");
        scene
            .set_row_input_number(&cards(), key.as_bytes(), "step", step)
            .expect("the step is written");
    });
}

/// Play `frames` frames: each report of each frame's step result, with the
/// frame it came in.
fn play(played: &mut Played, frames: usize) -> Vec<(usize, Heard)> {
    let mut events = Vec::new();
    for frame in 1..=frames {
        let output = played.step(FRAME, vec![]);
        events.extend(output.events.iter().map(|event| (frame, report(event))));
    }
    events
}

fn refused<T: std::fmt::Debug>(result: Result<T, Error>) -> ErrorKind {
    result.expect_err("refused").kind
}

fn step_running(played: &Played) -> bool {
    hosted_scene::root_scene(&played.board.native_handle())
        .expect("a page")
        .plan
        .borrow()
        .step_running()
}

fn begin_row(played: &mut Played, key: &str) -> Result<(), Error> {
    played
        .player
        .step_begin_row(&mut played.board, key.as_bytes(), &[], None)
}

fn end_row(played: &mut Played, key: &str) -> Result<(), Error> {
    played
        .player
        .step_end_row(&mut played.board, key.as_bytes(), &[], None)
}

fn poisoned(played: &Played) -> bool {
    crate::page_player::is_poisoned(&played.board.native_handle())
}

/// Proof 1 and 4. A driver writes the entering phase, step 7 and
/// reduceMotion false into row "b" by list and key. The enter layer starts
/// in that step, from the settle of the write's round. Thirty frames later
/// it fires "entered" once, which the step result and the driver both hear
/// with the list's path and "b"'s key. Held, it does not fire again.
/// Writing a value the input holds is no change and asks no further round.
/// Changing the step, and then the phase, returns the layer to idle.
#[test]
fn a_phase_written_into_a_row_enters_in_that_step_and_reports_once_with_its_key() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("b", CARD)]);
    let host = Host::on(&played);
    assert_eq!(playing(&played, "b"), 0);
    host.write(|scene| {
        let b = b"b".as_slice();
        assert_eq!(
            scene.set_row_input_number(&cards(), b, "phase", PHASE_ENTERING),
            Ok(true)
        );
        assert_eq!(
            scene.set_row_input_number(&cards(), b, "step", STEP),
            Ok(true)
        );
        assert_eq!(
            scene.set_row_input_bool(&cards(), b, "reduceMotion", false),
            Ok(false),
            "the file's value"
        );
        assert_eq!(scene.changes(), 2);
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, 2);
    assert!(!output.settled);
    assert_eq!(
        playing(&played, "b"),
        1,
        "the enter layer starts in the step"
    );
    assert_eq!(playing(&played, "a"), 0);
    host.take();

    let events = play(&mut played, 40);
    let path = row_path(&played);
    assert_eq!(
        events
            .iter()
            .map(|(_, heard)| heard.clone())
            .collect::<Vec<_>>(),
        [("entered".to_owned(), path.clone(), vec!["b".to_owned()])]
    );
    let frame = events[0].0;
    assert!((29..=31).contains(&frame), "after 30 frames: {frame}");
    assert_eq!(
        host.take(),
        [("entered".to_owned(), path, vec!["b".to_owned()])],
        "the driver hears it in the same step"
    );
    assert!(near(dot_x(&played, "b"), fixture::DOT_TRAVEL));
    assert_eq!(dot_x(&played, "a"), 0.0);
    assert_eq!(playing(&played, "b"), 1, "the layer holds the phase");
    assert!(play(&mut played, 60).is_empty(), "held, it fires no more");

    // Proof 4: the same value is no change, and no second round runs.
    host.write(|scene| {
        assert_eq!(
            scene.set_row_input_number(&cards(), b"b", "phase", PHASE_ENTERING),
            Ok(false)
        );
        assert_eq!(scene.changes(), 0);
    });
    assert_eq!(played.step(FRAME, vec![]).driver_rounds, 1);

    host.write(|scene| {
        assert_eq!(
            scene.set_row_input_number(&cards(), b"b", "step", STEP + 1.0),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    assert_eq!(playing(&played, "b"), 0, "a new step returns it to idle");
    write_phase(&host, "b", PHASE_ENTERING, STEP);
    played.step(FRAME, vec![]);
    assert_eq!(playing(&played, "b"), 1, "it enters again for step 7");
    write_phase(&host, "b", 0.0, STEP);
    played.step(FRAME, vec![]);
    assert_eq!(playing(&played, "b"), 0, "a new phase returns it to idle");
    assert!(play(&mut played, 40).is_empty());
}

/// Proof 2 (UNIV-7113). Two rows made from one artboard: the exit phase on
/// "a" and the enter phase on "b", in one round. Each plays its own layer
/// and reports with its own key; the rows don't cross.
#[test]
fn two_rows_of_one_artboard_play_their_own_phases_and_report_their_own_keys() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("b", CARD)]);
    let host = Host::on(&played);
    write_phase(&host, "a", PHASE_EXITING, STEP);
    write_phase(&host, "b", PHASE_ENTERING, STEP);
    played.step(FRAME, vec![]);
    let mut events = play(&mut played, 40)
        .into_iter()
        .map(|(_, heard)| (heard.0, heard.2))
        .collect::<Vec<_>>();
    events.sort();
    assert_eq!(
        events,
        [
            ("entered".to_owned(), vec!["b".to_owned()]),
            ("exited".to_owned(), vec!["a".to_owned()]),
        ]
    );
    assert!(near(dot_y(&played, "a"), fixture::DOT_TRAVEL));
    assert_eq!(dot_x(&played, "a"), 0.0);
    assert!(near(dot_x(&played, "b"), fixture::DOT_TRAVEL));
    assert_eq!(dot_y(&played, "b"), 0.0);
}

/// Proof 3. With reduceMotion set, the entering phase enters no layer and
/// fires no event.
#[test]
fn reduced_motion_enters_no_layer_and_fires_nothing() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD)]);
    let host = Host::on(&played);
    host.write(|scene| {
        assert_eq!(
            scene.set_row_input_bool(&cards(), b"a", "reduceMotion", true),
            Ok(true)
        );
    });
    write_phase(&host, "a", PHASE_ENTERING, STEP);
    played.step(FRAME, vec![]);
    assert_eq!(phase(&played, "a"), PHASE_ENTERING);
    assert_eq!(playing(&played, "a"), 0);
    assert!(play(&mut played, 40).is_empty());
    assert!(host.take().is_empty());
    assert_eq!(dot_x(&played, "a"), 0.0);
}

/// Proof 5. Refusals change nothing: a list path that leads nowhere, an
/// object that is not a keyed list, a key not mounted, a key removed this
/// round, an unknown name, a name of the other kind, a banner row whose
/// artboard has no machine, and a number that is not finite. A row mounted
/// in the round resolves, its machine made at mount.
#[test]
fn row_input_refusals_change_nothing_and_a_row_mounted_this_round_resolves() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("banner", BANNER)]);
    let host = Host::on(&played);
    host.write(|scene| {
        let number = |scene: &mut SceneWriter<'_>, list: &SceneObject, key: &[u8], name, value| {
            scene.set_row_input_number(list, key, name, value)
        };
        assert_eq!(
            number(scene, &SceneObject::root(999), b"a", "phase", 1.0),
            Err(SceneWriteError::NotFound)
        );
        assert_eq!(
            number(scene, &SceneObject::root(page::HEADING), b"a", "phase", 1.0),
            Err(SceneWriteError::NotKeyedList)
        );
        assert_eq!(
            number(scene, &cards(), b"gone", "phase", 1.0),
            Err(SceneWriteError::KeyNotFound)
        );
        assert_eq!(
            number(scene, &cards(), b"a", "nope", 1.0),
            Err(SceneWriteError::UnknownInput)
        );
        assert_eq!(
            number(scene, &cards(), b"a", "reduceMotion", 1.0),
            Err(SceneWriteError::UnknownInput)
        );
        assert_eq!(
            scene.set_row_input_bool(&cards(), b"a", "phase", true),
            Err(SceneWriteError::UnknownInput)
        );
        assert_eq!(
            number(scene, &cards(), b"banner", "phase", 1.0),
            Err(SceneWriteError::UnknownInput)
        );
        for value in [f32::NAN, f32::INFINITY] {
            assert_eq!(
                number(scene, &cards(), b"a", "phase", value),
                Err(SceneWriteError::InvalidValue)
            );
        }
        assert_eq!(scene.changes(), 0, "the refusals changed nothing");
        assert_eq!(scene.mount_row_from(&cards(), b"c", CARD), Ok(true));
        assert_eq!(
            number(scene, &cards(), b"c", "phase", PHASE_ENTERING),
            Ok(true)
        );
        assert_eq!(scene.remove_row(&cards(), b"a"), Ok(true));
        assert_eq!(
            number(scene, &cards(), b"a", "phase", 1.0),
            Err(SceneWriteError::KeyNotFound),
            "a key removed this round"
        );
        assert_eq!(scene.changes(), 3);
    });
    played.step(FRAME, vec![]);
    assert_eq!(keys(&played), ["banner", "c"]);
    assert_eq!(phase(&played, "c"), PHASE_ENTERING);
}

/// Proof 6. A sleeping row keeps the value written into it through many
/// frames, plays nothing and reports nothing, while an awake row written in
/// the same way enters at once; woken, the sleeping row enters and reports
/// with its key. The card's artboard holds an advancing component that is
/// never quiet, so its row never goes quiet, and the wake of a quiet row is
/// not shown with this fixture.
#[test]
fn a_sleeping_row_holds_a_written_phase_and_enters_when_it_wakes() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("b", CARD)]);
    let host = Host::on(&played);
    write_phase(&host, "a", PHASE_ENTERING, STEP);
    played.step(FRAME, vec![]);
    assert_eq!(playing(&played, "a"), 1);

    played
        .player
        .set_keyed_row_collapsed(&mut played.board, "Cards", b"b", true, &[], None)
        .expect("b sleeps");
    write_phase(&host, "b", PHASE_ENTERING, STEP);
    let events = play(&mut played, 60);
    assert_eq!(
        events
            .iter()
            .map(|(_, heard)| heard.2.clone())
            .collect::<Vec<_>>(),
        [vec!["a".to_owned()]],
        "only the awake row reports"
    );
    assert_eq!(
        phase(&played, "b"),
        PHASE_ENTERING,
        "the sleeping row holds it"
    );
    assert_eq!(playing(&played, "b"), 0);
    assert_eq!(dot_x(&played, "b"), 0.0);
    host.take();

    played
        .player
        .set_keyed_row_collapsed(&mut played.board, "Cards", b"b", false, &[], None)
        .expect("b wakes");
    let events = play(&mut played, 40);
    assert_eq!(
        events
            .into_iter()
            .map(|(_, heard)| (heard.0, heard.2))
            .collect::<Vec<_>>(),
        [("entered".to_owned(), vec!["b".to_owned()])]
    );
    assert!(near(dot_x(&played, "b"), fixture::DOT_TRAVEL));
}

/// Proof 7. A nameless step on a driver page names its arriving row by
/// key: begin marks the step running and end clears it. A sleeping row, a
/// key not mounted, a second begin and an end naming another row are
/// refused, and leave the occurrence usable. The name forms stay refused.
/// Removing the arriving row mid-step cancels the step in that step.
#[test]
fn a_row_step_runs_from_begin_to_end_and_cancels_when_its_row_leaves() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("b", CARD)]);
    begin_row(&mut played, "b").expect("the row step begins");
    assert!(step_running(&played));
    played.step(FRAME, vec![]);
    assert!(step_running(&played));
    assert_eq!(
        refused(begin_row(&mut played, "a")),
        ErrorKind::InvalidArgument
    );
    assert_eq!(
        refused(end_row(&mut played, "a")),
        ErrorKind::InvalidArgument
    );
    assert_eq!(
        refused(end_row(&mut played, "gone")),
        ErrorKind::InvalidArgument
    );
    assert!(step_running(&played));
    end_row(&mut played, "b").expect("the row step ends");
    assert!(!step_running(&played));
    assert_eq!(
        refused(end_row(&mut played, "b")),
        ErrorKind::InvalidArgument
    );

    assert_eq!(
        refused(begin_row(&mut played, "gone")),
        ErrorKind::InvalidArgument
    );
    played
        .player
        .set_keyed_row_collapsed(&mut played.board, "Cards", b"a", true, &[], None)
        .expect("a sleeps");
    assert_eq!(
        refused(begin_row(&mut played, "a")),
        ErrorKind::InvalidArgument
    );
    let begin = played
        .player
        .step_begin(&mut played.board, "Plan card", None, &[], None);
    assert_eq!(refused(begin), ErrorKind::InvalidArgument);
    let end = played
        .player
        .step_end(&mut played.board, "Plan card", None, &[], None);
    assert_eq!(refused(end), ErrorKind::InvalidArgument);
    assert!(!step_running(&played));
    assert!(!poisoned(&played));

    begin_row(&mut played, "b").expect("the row step begins");
    let host = Host::on(&played);
    host.write(|scene| {
        assert_eq!(scene.remove_row(&cards(), b"b"), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert!(!step_running(&played), "the step cancels in that step");
    assert_eq!(
        refused(end_row(&mut played, "b")),
        ErrorKind::InvalidArgument
    );
    assert!(!poisoned(&played));
    played.step(FRAME, vec![]);
}

/// The amendment's proof. The arriving row is held by its list item, not
/// its key: a key removed and mounted again in one round is a new row, so
/// the step cancels and an end naming that key does not match it.
#[test]
fn a_key_removed_and_mounted_again_in_one_round_is_a_new_row() {
    let mut played = page_with(Cards::PagePhases, &[("a", CARD), ("b", CARD)]);
    let old = machine(&played, "b");
    begin_row(&mut played, "b").expect("the row step begins");
    let host = Host::on(&played);
    host.write(|scene| {
        assert_eq!(scene.remove_row(&cards(), b"b"), Ok(true));
        assert_eq!(scene.mount_row_from(&cards(), b"b", CARD), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert_eq!(keys(&played), ["a", "b"]);
    assert!(
        !machine(&played, "b").downgrade().ptr_eq(&old.downgrade()),
        "a new row"
    );
    assert!(!step_running(&played), "the step cancels");
    assert_eq!(
        refused(end_row(&mut played, "b")),
        ErrorKind::InvalidArgument
    );
    assert!(!poisoned(&played));
    begin_row(&mut played, "b").expect("the new row can arrive");
    end_row(&mut played, "b").expect("and land");
}

/// Proof 7 and 8. On a keyed list that is not a page's rows, the input write
/// works as on a driver page and keeps no scene state, and the row step is
/// refused.
#[test]
fn a_row_input_write_needs_no_page_and_a_row_step_needs_a_driver_page() {
    let before = hosted_scene::bookkeeping_counts();
    let mut played = page_with(Cards::Phases, &[("a", CARD)]);
    let host = Host::on(&played);
    write_phase(&host, "a", PHASE_ENTERING, STEP);
    played.step(FRAME, vec![]);
    assert_eq!(playing(&played, "a"), 1);
    let events = play(&mut played, 40);
    assert_eq!(
        events
            .into_iter()
            .map(|(_, heard)| (heard.0, heard.2))
            .collect::<Vec<_>>(),
        [("entered".to_owned(), vec!["a".to_owned()])]
    );
    assert!(!hosted_scene::enabled(&played.board.native_handle()));
    assert_eq!(
        refused(begin_row(&mut played, "a")),
        ErrorKind::InvalidArgument
    );
    assert_eq!(
        refused(end_row(&mut played, "a")),
        ErrorKind::InvalidArgument
    );
    assert!(!poisoned(&played));
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}
