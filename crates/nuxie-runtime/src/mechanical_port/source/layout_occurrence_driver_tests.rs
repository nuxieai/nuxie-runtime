//! A driver writing inside one use of an artboard the page uses twice
//! (`occurrence_fixture`): the file has no view model and no binds, and every
//! write names its object by a path of nested artboard hosts and a local id.
//!
//! Oracle: the other use, which nothing wrote, and the CSS block flow, where
//! a box that grows or hides moves everything after it by its height.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::*;
use crate::RuntimeGeometryHitPathSegment as PathStep;
use crate::occurrence_fixture::{self as fixture, PageMachine, card, page};
use crate::source::nested_artboard_layout::NestedArtboardLayout;
use crate::source::semantic::semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager};
use crate::source::text::text_input::TextInput;

/// Long enough to wrap in the card's 100 point label.
const LONG: &str = "Monthly plan, billed every month";

fn two_uses(machine: PageMachine) -> Scene {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::two_uses_fixture(machine),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the two uses fixture imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("the page artboard");
    Scene {
        _file: file,
        artboard,
        factory,
    }
}

/// The page played on its state machine, settled.
fn played(machine: PageMachine) -> Played {
    let mut played = Played::new(two_uses(machine));
    played.frames_until_settled();
    played
}

/// The page played with no state machine, settled.
fn played_static() -> Played {
    let f = two_uses(PageMachine::None);
    let board = ArtboardInstance::from_native_handle(f._file.clone(), 0, f.artboard.clone());
    let mut played = Played {
        f,
        board,
        playback: Playback::StaticArtboard,
        player: PagePlayer::default(),
    };
    played.step(0.0, vec![]);
    played.frames_until_settled();
    played
}

/// The path to the use hosted by `host`, as a report carries it.
fn path_to(played: &Played, host: usize) -> Vec<PathStep> {
    vec![PathStep {
        artboard_global_id: crate::host_semantics::source_artboard_global_id(&played.f.artboard)
            .expect("the page's global id"),
        local_id: host,
    }]
}

fn in_use(played: &Played, host: usize, local_id: usize) -> SceneObject {
    SceneObject::nested(path_to(played, host), local_id)
}

fn host_name(host: usize) -> &'static str {
    match host {
        page::FIRST => "First",
        page::SECOND => "Second",
        _ => panic!("no use at {host}"),
    }
}

/// The card artboard the host at `host` holds.
fn card_of(played: &Played, host: usize) -> RuntimeArtboardInstanceHandle {
    played
        .f
        .object(host_name(host))
        .with(|host| {
            host.as_nested_artboard()
                .and_then(|nested| nested.artboard_instance_default())
        })
        .flatten()
        .expect("the hosted card")
}

/// The object at `local_id` in the use hosted by `host`.
fn card_object(played: &Played, host: usize, local_id: usize) -> CoreHandle {
    card_of(played, host)
        .with_artboard(|card| card.objects().get(local_id).cloned().flatten())
        .expect("an object of the card")
}

/// A host's top in the page.
fn host_top(played: &Played, host: usize) -> f32 {
    played
        .f
        .object(host_name(host))
        .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
        .expect("a layout host")
        .top()
}

/// A card box's left, top, width and height, in its card.
fn card_box(played: &Played, host: usize, local_id: usize) -> [f32; 4] {
    let bounds = card_object(played, host, local_id)
        .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds())
        .expect("a card box");
    [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
}

fn run_text(played: &Played, host: usize) -> String {
    CoreRegistry::get_string_handle(
        &card_object(played, host, card::RUN),
        i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
    )
    .expect("the run's text")
}

fn field_text(played: &Played, host: usize) -> String {
    use crate::source::generated::text::text_input_base::TextInputBase;
    CoreRegistry::get_string_handle(
        &card_object(played, host, card::FIELD),
        i32::from(TextInputBase::TEXT_PROPERTY_KEY),
    )
    .expect("the field's text")
}

fn dot_x(played: &Played, host: usize) -> f32 {
    CoreRegistry::get_double_handle(
        &card_object(played, host, card::DOT),
        i32::from(NodeBase::X_PROPERTY_KEY),
    )
    .expect("the dot's x")
}

/// The middle of a use's button, in the page.
fn button_center(played: &Played, host: usize) -> (f32, f32) {
    let [left, top, width, height] = card_box(played, host, card::BUTTON);
    (
        left + width / 2.0,
        host_top(played, host) + top + height / 2.0,
    )
}

/// A driver that runs `write` once, in the first round of the next step.
fn write_once(played: &Played, write: impl FnOnce(&mut SceneWriter<'_>) + 'static) {
    let write = RefCell::new(Some(write));
    played.drive(move |round, scene| {
        if round.index == 0
            && let Some(write) = write.borrow_mut().take()
        {
            write(scene);
        }
    });
}

/// A text written in one use shows there and not in the other, and the page
/// lays out around it in the step of the write: the wrapped label grows the
/// first card, so the second use and the box after it move down by as much.
#[test]
fn a_text_written_in_one_use_lays_out_the_page_in_the_same_step() {
    let mut played = played(PageMachine::OneLayer);
    let second = host_top(&played, page::SECOND);
    let below = played.f.bounds("Below")[1];
    let label = card_box(&played, page::FIRST, card::LABEL)[3];
    let run = in_use(&played, page::FIRST, card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, LONG),
            Ok(true)
        );
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, 2);
    assert_eq!(run_text(&played, page::FIRST), LONG);
    assert_eq!(run_text(&played, page::SECOND), fixture::RUN_TEXT);
    let grown = card_box(&played, page::FIRST, card::LABEL)[3] - label;
    assert!(grown > 10.0, "the label wraps: {grown}");
    assert_eq!(
        card_box(&played, page::SECOND, card::LABEL)[3],
        label,
        "the other use's label"
    );
    assert!(
        near(host_top(&played, page::SECOND), second + grown),
        "the second use moved in the step: {} from {second}",
        host_top(&played, page::SECOND)
    );
    assert!(
        near(played.f.bounds("Below")[1], below + grown),
        "the box after it moved in the step: {} from {below}",
        played.f.bounds("Below")[1]
    );
}

/// `display: none` written on a box inside one use hides it in the step:
/// the box after it in that card moves up, and so do the second use and the
/// page's box after it, while the second use's own box stays.
#[test]
fn a_display_written_in_one_use_hides_it_and_moves_what_follows() {
    let mut played = played(PageMachine::OneLayer);
    let second = host_top(&played, page::SECOND);
    let below = played.f.bounds("Below")[1];
    let field = card_box(&played, page::FIRST, card::FIELD_BOX)[1];
    let record = in_use(&played, page::FIRST, card::BUTTON_RECORD);
    write_once(&played, move |scene| {
        assert_eq!(scene.set_unsigned(&record, DISPLAY, NONE as u32), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert!(near(
        card_box(&played, page::FIRST, card::FIELD_BOX)[1],
        field - 40.0
    ));
    assert!(near(host_top(&played, page::SECOND), second - 40.0));
    assert!(near(played.f.bounds("Below")[1], below - 40.0));
    assert!(near(
        card_box(&played, page::SECOND, card::FIELD_BOX)[1],
        field
    ));
    assert!(near(card_box(&played, page::SECOND, card::BUTTON)[3], 40.0));
}

/// A tap on one use's button arrives as a report whose path, handed back
/// with a local id of the card, names that use: a write through it lands
/// there and not in the other use.
#[test]
fn a_reported_path_handed_back_writes_the_tapped_use() {
    let mut played = played(PageMachine::Listening);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    played.drive(move |round, scene| {
        for report in round.reports {
            log.borrow_mut().push(report.clone());
            let path = report.context.as_ref().expect("a path").path().to_vec();
            let run = SceneObject::nested(path, card::RUN);
            assert_eq!(
                scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
                Ok(true)
            );
        }
    });
    let (x, y) = button_center(&played, page::SECOND);
    played.tap(x, y);
    let heard = heard.borrow();
    assert_eq!(names(&heard), ["chosen"]);
    assert_eq!(heard[0].listener_target_local_id, Some(card::BUTTON));
    assert_eq!(
        heard[0].context.as_ref().expect("a path").path(),
        path_to(&played, page::SECOND)
    );
    assert_eq!(run_text(&played, page::SECOND), "Chosen");
    assert_eq!(run_text(&played, page::FIRST), fixture::RUN_TEXT);
}

/// What a tap on a use's button reports for each state machine the page
/// can have when the page hears nothing itself. The card's own listener
/// hears the tap through the page's machine, which hit-tests every nested
/// artboard's machine; a player without a machine hears no pointer.
#[test]
fn a_tap_reaches_a_use_through_a_page_machine_with_no_listener() {
    let mut heard = Vec::new();
    for machine in [PageMachine::OneLayer, PageMachine::Bare, PageMachine::None] {
        let mut played = if machine == PageMachine::None {
            played_static()
        } else {
            played(machine)
        };
        let paths = Rc::new(RefCell::new(Vec::new()));
        let log = paths.clone();
        played.drive(move |round, _| {
            log.borrow_mut().extend(round.reports.iter().map(|report| {
                (
                    names(std::slice::from_ref(report)),
                    report
                        .context
                        .as_ref()
                        .map(|context| context.path().to_vec()),
                )
            }));
        });
        let (x, y) = button_center(&played, page::FIRST);
        let output = played.tap(x, y);
        let first = path_to(&played, page::FIRST);
        heard.push((
            machine,
            names(&output.events),
            paths
                .borrow()
                .iter()
                .all(|(_, path)| path.as_ref() == Some(&first)),
        ));
    }
    assert_eq!(
        heard,
        [
            (PageMachine::OneLayer, vec!["chosen".to_owned()], true),
            (PageMachine::Bare, vec!["chosen".to_owned()], true),
            (PageMachine::None, vec![], true),
        ]
    );
}

/// A reset that names a host in the page returns that use's machine to its
/// entry, which plays "intro" again from its start; the other use's machine
/// keeps its state.
#[test]
fn a_reset_of_a_host_returns_that_uses_machine_only() {
    let mut played = played(PageMachine::OneLayer);
    assert!(near(dot_x(&played, page::FIRST), fixture::DOT_TRAVEL));
    assert!(near(dot_x(&played, page::SECOND), fixture::DOT_TRAVEL));
    let host = SceneObject::root(page::FIRST);
    write_once(&played, move |scene| {
        assert_eq!(scene.reset(&host), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert!(
        dot_x(&played, page::FIRST) < fixture::DOT_TRAVEL / 2.0,
        "restarted: {}",
        dot_x(&played, page::FIRST)
    );
    assert!(near(dot_x(&played, page::SECOND), fixture::DOT_TRAVEL));
    played.frames_until_settled();
    assert!(near(dot_x(&played, page::FIRST), fixture::DOT_TRAVEL));
}

/// A reset rooted inside a use returns only what is under that object: the
/// field it names takes back its written text, the other use's field keeps
/// what was typed, and the use's machine, which belongs to its host, keeps
/// its state.
#[test]
fn a_reset_inside_a_use_returns_only_what_is_under_it() {
    let mut played = played(PageMachine::OneLayer);
    for host in [page::FIRST, page::SECOND] {
        assert_eq!(
            card_object(&played, host, card::FIELD)
                .with_downcast_mut::<TextInput, _>(|input| input.text_input("!")),
            Some(true)
        );
        assert_ne!(field_text(&played, host), fixture::FIELD_TEXT);
    }
    let typed = field_text(&played, page::SECOND);
    let field_box = in_use(&played, page::FIRST, card::FIELD_BOX);
    write_once(&played, move |scene| {
        assert_eq!(scene.reset(&field_box), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert_eq!(field_text(&played, page::FIRST), fixture::FIELD_TEXT);
    assert_eq!(field_text(&played, page::SECOND), typed);
    assert!(near(dot_x(&played, page::FIRST), fixture::DOT_TRAVEL));
    assert!(near(dot_x(&played, page::SECOND), fixture::DOT_TRAVEL));
}

/// A path that leads nowhere is refused with every kind of write and with a
/// reset, and nothing changes: a step that names no object, an object that
/// hosts no nested artboard, an artboard other than the one the path has
/// reached, and a step past the use. A path that reaches a use, with a
/// local id the card does not have, is an unknown object.
#[test]
fn a_bad_path_is_refused_and_changes_nothing() {
    let mut played = played(PageMachine::OneLayer);
    let page_id = path_to(&played, page::FIRST)[0].artboard_global_id;
    let card_id = crate::host_semantics::source_artboard_global_id(&card_of(&played, page::FIRST))
        .expect("the card's global id");
    assert_ne!(page_id, card_id);
    let step = |artboard_global_id: u32, local_id: usize| PathStep {
        artboard_global_id,
        local_id,
    };
    let refused = vec![
        (vec![step(page_id, 999)], SceneWriteError::PathNotFound),
        (
            vec![step(page_id, page::BELOW)],
            SceneWriteError::PathNotFound,
        ),
        (
            vec![step(card_id, page::FIRST)],
            SceneWriteError::PathNotFound,
        ),
        (
            vec![step(page_id, page::FIRST), step(card_id, card::BUTTON)],
            SceneWriteError::PathNotFound,
        ),
        (vec![step(page_id, page::FIRST)], SceneWriteError::NotFound),
    ];
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    let asked = Rc::new(Cell::new(false));
    let once = asked.clone();
    played.drive(move |round, scene| {
        if round.index != 0 || once.replace(true) {
            return;
        }
        for (path, expected) in &refused {
            let missing = *expected == SceneWriteError::NotFound;
            let object = SceneObject::nested(path.clone(), if missing { 999 } else { card::RUN });
            let color =
                SceneObject::nested(path.clone(), if missing { 999 } else { card::RUN_COLOR });
            log.borrow_mut().push((
                *expected,
                [
                    scene.set_text(&object, TextValueRunBase::TEXT_PROPERTY_KEY, "x"),
                    scene.set_number(&object, NodeBase::X_PROPERTY_KEY, 1.0),
                    scene.set_unsigned(&object, DISPLAY, NONE as u32),
                    scene.set_bool(&object, 0, true),
                    scene.set_color(&color, SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0),
                    scene.reset(&object),
                ]
                .map(|answer| answer == Err(*expected)),
            ));
        }
        assert_eq!(scene.changes(), 0);
    });
    let output = played.step(FRAME, vec![]);
    assert!(asked.get());
    for (expected, answers) in answers.borrow().iter() {
        assert_eq!(answers, &[true; 6], "{expected:?}");
    }
    assert_eq!(output.driver_rounds, 1, "no round after a refused write");
    for host in [page::FIRST, page::SECOND] {
        assert_eq!(run_text(&played, host), fixture::RUN_TEXT);
    }
}

/// The empty path is the player's own artboard, as before paths: a write
/// names the page's object at that local id, never a use's.
#[test]
fn an_empty_path_writes_the_page() {
    let mut played = played(PageMachine::OneLayer);
    let below = SceneObject::root(page::BELOW_RECORD);
    write_once(&played, move |scene| {
        assert_eq!(scene.set_number(&below, WIDTH_KIND + 1, 120.0), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert!(near(played.f.bounds("Below")[2], 120.0));
    for host in [page::FIRST, page::SECOND] {
        assert_eq!(card_box(&played, host, card::LABEL)[2], 100.0);
    }
}

/// The semantic tree holds each use's text and button at that use's frame
/// in the page, and a text written in one use shows in that use's node only.
#[test]
fn the_semantic_snapshot_holds_each_use_at_its_frame() {
    let mut played = Played::new(two_uses(PageMachine::OneLayer));
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::new());
    played
        .f
        .artboard
        .build_semantic_tree(Some(manager.clone()), None);
    played.frames_until_settled();
    let run = in_use(&played, page::FIRST, card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Yearly"),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let nodes = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    let mut texts: Vec<_> = nodes
        .iter()
        .filter(|node| node.role == 7)
        .map(|node| (node.label.clone(), node.min_y))
        .collect();
    texts.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut buttons: Vec<_> = nodes
        .iter()
        .filter(|node| node.role == 1)
        .map(|node| (node.label.clone(), node.min_x, node.min_y, node.max_y))
        .collect();
    buttons.sort_by(|a, b| a.2.total_cmp(&b.2));
    let tops = [page::FIRST, page::SECOND].map(|host| host_top(&played, host));
    assert_eq!(
        texts
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>(),
        ["Yearly", fixture::RUN_TEXT],
        "{nodes:#?}"
    );
    for ((_, top), host) in texts.iter().zip(tops) {
        assert!(*top >= host - 0.05 && *top < host + 5.0, "{top} in {host}");
    }
    assert_eq!(buttons.len(), 2, "{nodes:#?}");
    for ((label, left, top, bottom), host) in buttons.iter().zip([page::FIRST, page::SECOND]) {
        let [x, y, _, height] = card_box(&played, host, card::BUTTON);
        assert_eq!(label, fixture::BUTTON_LABEL);
        assert!(near(*left, x), "{left} and {x}");
        assert!(near(*top, host_top(&played, host) + y), "{top} in {host}");
        assert!(near(*bottom - *top, height));
    }
}
