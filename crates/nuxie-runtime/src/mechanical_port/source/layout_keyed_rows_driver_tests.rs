//! A driver mounting, removing and moving the rows of a keyed list by key
//! (`keyed_rows_fixture`): the file has no view model and no binds, the list's
//! record names the card artboard its rows are made from, and every write
//! into a row names the row by the list's host object and its key.
//!
//! Oracle: the CSS block flow, where the rows sit in the column between the
//! heading and the footer, one after another, and a row that grows moves
//! everything after it by as much; the other rows, which nothing wrote; and
//! the card artboard's file values for a row made fresh.
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::keyed_rows_fixture::{self as fixture, Cards, badge, card, page};
use crate::source::artboard::Artboard;
use crate::source::artboard_component_list::ArtboardComponentList;
use crate::source::semantic::semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager};
use crate::source::viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem;
use crate::{RuntimeGeometryHitPathSegment as PathStep, RuntimeListRowKey};

/// Long enough to wrap in the card's 140 point label.
const LONG: &str = "Monthly plan, billed every month, cancel any time";

fn import(bytes: &[u8]) -> Scene {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the plans fixture imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("the plans artboard");
    Scene {
        _file: file,
        artboard,
        factory,
    }
}

/// The plans screen played on its state machine, settled, with no rows.
fn played(cards: Cards) -> Played {
    let mut played = Played::new(import(&fixture::keyed_plans_fixture(cards)));
    played.frames_until_settled();
    played
}

/// The plans screen with a row mounted for each key, settled.
fn played_with(keys: &[&'static str]) -> Played {
    let mut played = played(Cards::Keyed);
    mount(&mut played, keys);
    played.frames_until_settled();
    played
}

fn page_id(played: &Played) -> u32 {
    crate::host_semantics::source_artboard_global_id(&played.f.artboard)
        .expect("the page's global id")
}

/// The "Cards" list as a driver names it.
fn cards() -> SceneObject {
    SceneObject::root(page::CARDS)
}

/// The path to the "Cards" list's rows, as a report carries it.
fn row_path(played: &Played) -> Vec<PathStep> {
    vec![PathStep {
        artboard_global_id: page_id(played),
        local_id: page::CARDS,
    }]
}

/// The row key of the "Cards" list's row mounted by `key`.
fn row_key(played: &Played, key: &str) -> RuntimeListRowKey {
    RuntimeListRowKey {
        artboard_global_id: page_id(played),
        host_local_id: page::CARDS,
        key: key.as_bytes().into(),
    }
}

/// The object at `local_id` in the row mounted by `key`, as a driver names it.
fn in_row(played: &Played, key: &str, local_id: usize) -> SceneObject {
    SceneObject::in_rows(row_path(played), vec![row_key(played, key)], local_id)
}

fn list(played: &Played) -> CoreHandle {
    played
        .f
        .artboard
        .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone())
}

/// The keys of the list's rows, in order.
fn keys(played: &Played) -> Vec<String> {
    list(played)
        .with_downcast::<ArtboardComponentList, _>(|list| {
            list.row_keys()
                .into_iter()
                .map(|key| String::from_utf8_lossy(key).into_owned())
                .collect()
        })
        .expect("the list")
}

/// The keys of the rows the list lays out, in order.
fn laid_out_keys(played: &Played) -> Vec<String> {
    list(played)
        .with_downcast::<ArtboardComponentList, _>(|list| {
            (0..list.artboard_count() as i32)
                .map(|index| {
                    let item = list.list_item(index).expect("an item");
                    String::from_utf8_lossy(list.key_of_row(&item).expect("a keyed row"))
                        .into_owned()
                })
                .collect()
        })
        .expect("the list")
}

/// The card artboard of the row mounted by `key`.
fn row_of(played: &Played, key: &str) -> RuntimeArtboardInstanceHandle {
    list(played)
        .with_downcast::<ArtboardComponentList, _>(|list| list.keyed_row(key.as_bytes()))
        .flatten()
        .expect("a mounted row")
        .1
}

fn row_object(played: &Played, key: &str, local_id: usize) -> CoreHandle {
    row_of(played, key)
        .with_artboard(|row| row.objects().get(local_id).cloned().flatten())
        .expect("an object of the card")
}

/// A row's top and height in the page.
fn row_frame(played: &Played, key: &str) -> (f32, f32) {
    row_of(played, key).with_artboard(|row| (row.layout_y(), row.layout_bounds().height()))
}

/// A card box's left, top, width and height, in its row.
fn card_box(played: &Played, key: &str, local_id: usize) -> [f32; 4] {
    let bounds = row_object(played, key, local_id)
        .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds())
        .expect("a card box");
    [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
}

fn run_text(played: &Played, key: &str) -> String {
    CoreRegistry::get_string_handle(
        &row_object(played, key, card::RUN),
        i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
    )
    .expect("the run's text")
}

fn scroll_y(played: &Played, key: &str) -> f32 {
    CoreRegistry::get_double_handle(
        &row_object(played, key, card::SCROLL),
        i32::from(fixture::SCROLL_OFFSET_Y),
    )
    .expect("the scroll offset")
}

fn dot_x(played: &Played, key: &str) -> f32 {
    CoreRegistry::get_double_handle(
        &row_object(played, key, card::DOT),
        i32::from(NodeBase::X_PROPERTY_KEY),
    )
    .expect("the dot's x")
}

/// The middle of a row's button, in the page.
fn button_center(played: &Played, key: &str) -> (f32, f32) {
    let [left, top, width, height] = card_box(played, key, card::BUTTON);
    let (x, y) = row_of(played, key).with_artboard(|row| (row.layout_x(), row.layout_y()));
    (x + left + width / 2.0, y + top + height / 2.0)
}

fn footer_top(played: &Played) -> f32 {
    played.f.bounds("Footer")[1]
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

/// Mount a row for each key, in one round of one step.
fn mount(played: &mut Played, keys: &[&'static str]) {
    let keys = keys.to_vec();
    write_once(played, move |scene| {
        for key in keys {
            assert_eq!(scene.mount_row(&cards(), key.as_bytes()), Ok(true));
        }
    });
    played.step(FRAME, vec![]);
}

fn pointer(kind: PointerKind, (x, y): (f32, f32)) -> Pointer {
    Pointer {
        kind,
        x,
        y,
        pointer_id: 0,
        timestamp_seconds: 0.0,
    }
}

/// Three rows mounted sit one after another between the heading and the
/// footer in the step of the mount. A text written in one row shows there
/// only, and the page lays out around it in the step of the write: the
/// wrapped label grows that row, so the row after it and the footer move
/// down by as much.
#[test]
fn three_rows_mount_and_a_text_written_in_one_lays_out_the_page() {
    let mut played = played(Cards::Keyed);
    assert!(near(footer_top(&played), fixture::HEADING_HEIGHT));
    let keys_mounted = ["monthly", "yearly", "lifetime"];
    let keys_list = keys_mounted.to_vec();
    write_once(&played, move |scene| {
        for key in keys_list {
            assert_eq!(scene.mount_row(&cards(), key.as_bytes()), Ok(true));
        }
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, 2);
    assert_eq!(keys(&played), keys_mounted);
    assert_eq!(laid_out_keys(&played), keys_mounted);
    let (top, height) = row_frame(&played, "monthly");
    assert!(height > 100.0, "a card's height: {height}");
    assert!(near(top, fixture::HEADING_HEIGHT), "{top}");
    assert!(near(
        footer_top(&played),
        fixture::HEADING_HEIGHT + 3.0 * height
    ));
    played.frames_until_settled();

    let lifetime = row_frame(&played, "lifetime").0;
    let footer = footer_top(&played);
    let label = card_box(&played, "yearly", card::LABEL)[3];
    let run = in_row(&played, "yearly", card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, LONG),
            Ok(true)
        );
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, 2);
    assert_eq!(run_text(&played, "yearly"), LONG);
    for key in ["monthly", "lifetime"] {
        assert_eq!(run_text(&played, key), fixture::RUN_TEXT);
        assert!(near(card_box(&played, key, card::LABEL)[3], label));
    }
    let grown = card_box(&played, "yearly", card::LABEL)[3] - label;
    assert!(grown > 10.0, "the label wraps: {grown}");
    assert!(
        near(row_frame(&played, "lifetime").0, lifetime + grown),
        "the row after it moved in the step"
    );
    assert!(
        near(footer_top(&played), footer + grown),
        "the footer moved"
    );
}

/// A row moved keeps the text written into it and its scroll offset, and
/// sits at its new place in the step of the move.
#[test]
fn a_moved_row_keeps_its_text_and_scroll() {
    let mut played = played_with(&["monthly", "yearly", "lifetime"]);
    let run = in_row(&played, "monthly", card::RUN);
    let scroll = in_row(&played, "monthly", card::SCROLL);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
            Ok(true)
        );
        assert_eq!(
            scene.set_number(&scroll, fixture::SCROLL_OFFSET_Y, -50.0),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    assert_eq!(scroll_y(&played, "monthly"), -50.0);
    let height = row_frame(&played, "yearly").1;
    write_once(&played, move |scene| {
        assert_eq!(scene.move_row(&cards(), b"monthly", 2), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert_eq!(keys(&played), ["yearly", "lifetime", "monthly"]);
    assert_eq!(laid_out_keys(&played), ["yearly", "lifetime", "monthly"]);
    assert!(near(
        row_frame(&played, "yearly").0,
        fixture::HEADING_HEIGHT
    ));
    assert!(near(
        row_frame(&played, "monthly").0,
        fixture::HEADING_HEIGHT + 2.0 * height
    ));
    played.frames_until_settled();
    assert_eq!(run_text(&played, "monthly"), "Chosen");
    assert_eq!(scroll_y(&played, "monthly"), -50.0);
    assert_eq!(scroll_y(&played, "yearly"), 0.0);
}

/// The first of four rows goes last as three moves in one round: every row
/// keeps its instance, and the list brings its rows up to date once.
#[test]
fn three_moves_in_a_round_bring_the_list_up_to_date_once() {
    let mut played = played_with(&["a", "b", "c", "d"]);
    let before: Vec<_> = ["a", "b", "c", "d"]
        .map(|key| row_of(&played, key))
        .to_vec();
    write_once(&played, move |scene| {
        for place in 1..=3 {
            assert_eq!(scene.move_row(&cards(), b"a", place), Ok(true));
        }
        assert_eq!(scene.changes(), 3);
    });
    crate::hosted_scene::measure_walks(true);
    let output = played.step(FRAME, vec![]);
    let walks = crate::hosted_scene::measure_walks(false);
    assert_eq!(output.driver_rounds, 2);
    assert_eq!(walks.list_rows, 1, "{walks:?}");
    assert_eq!(laid_out_keys(&played), ["b", "c", "d", "a"]);
    for (key, row) in ["a", "b", "c", "d"].into_iter().zip(&before) {
        assert!(
            row_of(&played, key).downgrade().ptr_eq(&row.downgrade()),
            "{key} keeps its instance"
        );
    }
    let height = row_frame(&played, "b").1;
    assert!(near(
        row_frame(&played, "a").0,
        fixture::HEADING_HEIGHT + 3.0 * height
    ));
}

/// One mount in a round brings the list up to date once. The walks of the
/// step are printed for the record.
#[test]
fn a_mount_brings_the_list_up_to_date_once() {
    let mut played = played_with(&["monthly"]);
    write_once(&played, |scene| {
        assert_eq!(scene.mount_row(&cards(), b"yearly"), Ok(true));
    });
    crate::hosted_scene::measure_walks(true);
    played.step(FRAME, vec![]);
    let walks = crate::hosted_scene::measure_walks(false);
    eprintln!("one mount's step: {walks:?}");
    assert_eq!(walks.list_rows, 1, "{walks:?}");
    assert_eq!(laid_out_keys(&played), ["monthly", "yearly"]);
}

/// A row removed and its key mounted again in the same round is a new row
/// with the file's values: the written text is gone, the scroll is at the
/// file's offset, and the machine starts over at its entry, playing "intro"
/// from its start.
#[test]
fn a_key_mounted_again_is_a_new_row_with_the_files_values() {
    let mut played = played_with(&["monthly", "yearly"]);
    assert!(near(dot_x(&played, "monthly"), fixture::DOT_TRAVEL));
    let run = in_row(&played, "monthly", card::RUN);
    let scroll = in_row(&played, "monthly", card::SCROLL);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
            Ok(true)
        );
        assert_eq!(
            scene.set_number(&scroll, fixture::SCROLL_OFFSET_Y, -50.0),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let old = row_of(&played, "monthly");
    write_once(&played, |scene| {
        assert_eq!(scene.remove_row(&cards(), b"monthly"), Ok(true));
        assert_eq!(scene.mount_row(&cards(), b"monthly"), Ok(true));
    });
    played.step(FRAME, vec![]);
    let new = row_of(&played, "monthly");
    assert!(!new.downgrade().ptr_eq(&old.downgrade()), "a new instance");
    assert_eq!(laid_out_keys(&played), ["yearly", "monthly"]);
    assert_eq!(run_text(&played, "monthly"), fixture::RUN_TEXT);
    assert_eq!(scroll_y(&played, "monthly"), 0.0);
    assert!(
        dot_x(&played, "monthly") < fixture::DOT_TRAVEL / 2.0,
        "the machine starts over: {}",
        dot_x(&played, "monthly")
    );
    played.frames_until_settled();
    assert!(near(dot_x(&played, "monthly"), fixture::DOT_TRAVEL));
    drop(old);
}

/// A tap on a row's button arrives as a report carrying the row's key beside
/// its path; the path and key handed back write that row and no other.
#[test]
fn a_tap_in_a_row_reports_its_key_and_the_key_handed_back_writes_it() {
    let mut played = played_with(&["monthly", "yearly"]);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    played.drive(move |round, scene| {
        for report in round.reports {
            log.borrow_mut().push(report.clone());
            let context = report.context.as_ref().expect("a context");
            let run =
                SceneObject::in_rows(context.path().to_vec(), context.rows().to_vec(), card::RUN);
            assert_eq!(
                scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
                Ok(true)
            );
        }
    });
    let center = button_center(&played, "yearly");
    played.tap(center.0, center.1);
    let heard = heard.borrow();
    assert_eq!(names(&heard), ["chosen"]);
    assert_eq!(heard[0].listener_target_local_id, Some(card::BUTTON));
    let context = heard[0].context.as_ref().expect("a context");
    assert_eq!(context.path(), row_path(&played));
    assert_eq!(context.rows(), [row_key(&played, "yearly")]);
    assert_eq!(run_text(&played, "yearly"), "Chosen");
    assert_eq!(run_text(&played, "monthly"), fixture::RUN_TEXT);
}

/// A text set into a row in the round that mounts it lands, and the page
/// lays out around it in that step: the label wraps, and the footer sits
/// under the grown row.
#[test]
fn a_set_into_a_row_mounted_in_the_same_round_lays_out_in_that_step() {
    let mut played = played_with(&["monthly"]);
    let (_, plain) = row_frame(&played, "monthly");
    let label = card_box(&played, "monthly", card::LABEL)[3];
    let run = in_row(&played, "yearly", card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(scene.mount_row(&cards(), b"yearly"), Ok(true));
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, LONG),
            Ok(true)
        );
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, 2);
    assert_eq!(run_text(&played, "yearly"), LONG);
    let grown = card_box(&played, "yearly", card::LABEL)[3] - label;
    assert!(grown > 10.0, "the label wraps in the step: {grown}");
    let (top, height) = row_frame(&played, "yearly");
    assert!(near(top, fixture::HEADING_HEIGHT + plain));
    assert!(near(height, plain + grown), "{height} {plain} {grown}");
    assert!(near(footer_top(&played), top + height));
}

/// Rows of a list between two siblings in a column sit at the list's place:
/// after the heading, one after another, and before the footer, which moves
/// down by each row mounted and back up by each removed.
#[test]
fn rows_sit_at_the_lists_place_between_its_siblings() {
    let mut played = played_with(&["monthly", "yearly"]);
    assert!(near(played.f.bounds("Heading")[1], 0.0));
    let (first, height) = row_frame(&played, "monthly");
    let (second, _) = row_frame(&played, "yearly");
    assert!(near(first, fixture::HEADING_HEIGHT));
    assert!(near(second, first + height));
    assert!(near(footer_top(&played), second + height));
    write_once(&played, |scene| {
        assert_eq!(scene.remove_row(&cards(), b"monthly"), Ok(true));
    });
    played.step(FRAME, vec![]);
    assert!(near(
        row_frame(&played, "yearly").0,
        fixture::HEADING_HEIGHT
    ));
    assert!(near(footer_top(&played), fixture::HEADING_HEIGHT + height));
}

/// A press held on a row's button while the row moves, then released where
/// the button was, reports nothing: the row now there never had the press.
/// A press held on a row while it is removed, then released, reports
/// nothing. No press is left stuck: a tap after them reports once, from the
/// row tapped, with its key.
#[test]
fn a_press_held_on_a_row_that_moves_or_goes_ends_cleanly() {
    let mut played = played_with(&["a", "b", "c"]);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    let next = Rc::new(RefCell::new(None::<Box<dyn FnOnce(&mut SceneWriter<'_>)>>));
    let write = next.clone();
    played.drive(move |round, scene| {
        log.borrow_mut().extend(round.reports.iter().cloned());
        if round.index == 0
            && let Some(write) = write.borrow_mut().take()
        {
            write(scene);
        }
    });

    let at_a = button_center(&played, "a");
    played.step(FRAME, vec![pointer(PointerKind::Down, at_a)]);
    *next.borrow_mut() = Some(Box::new(|scene| {
        assert_eq!(scene.move_row(&cards(), b"a", 2), Ok(true));
    }));
    played.step(FRAME, vec![]);
    assert_eq!(laid_out_keys(&played), ["b", "c", "a"]);
    played.step(FRAME, vec![pointer(PointerKind::Up, at_a)]);
    played.frames_until_settled();
    assert_eq!(names(&heard.borrow()), Vec::<String>::new());

    let at_b = button_center(&played, "b");
    played.step(FRAME, vec![pointer(PointerKind::Down, at_b)]);
    *next.borrow_mut() = Some(Box::new(|scene| {
        assert_eq!(scene.remove_row(&cards(), b"b"), Ok(true));
    }));
    played.step(FRAME, vec![]);
    assert_eq!(laid_out_keys(&played), ["c", "a"]);
    played.step(FRAME, vec![pointer(PointerKind::Up, at_b)]);
    played.frames_until_settled();
    assert_eq!(names(&heard.borrow()), Vec::<String>::new());

    let at_a = button_center(&played, "a");
    played.tap(at_a.0, at_a.1);
    let heard = heard.borrow();
    assert_eq!(names(&heard), ["chosen"]);
    assert_eq!(
        heard[0].context.as_ref().expect("a context").rows(),
        [row_key(&played, "a")]
    );
}

/// Each semantic text node, top to bottom.
fn semantic_texts(manager: &RuntimeSemanticManagerHandle) -> Vec<String> {
    let mut nodes: Vec<_> = manager
        .with_semantic_manager_mut(|manager| manager.snapshot().to_vec())
        .into_iter()
        .filter(|node| node.role == 7)
        .map(|node| (node.label, node.min_y))
        .collect();
    nodes.sort_by(|a, b| a.1.total_cmp(&b.1));
    nodes.into_iter().map(|(label, _)| label).collect()
}

/// The semantic snapshot lists the rows' texts in the rows' order, and in
/// their new order after a move.
#[test]
fn the_semantic_snapshot_follows_a_move() {
    let mut played = Played::new(import(&fixture::keyed_plans_fixture(Cards::Keyed)));
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::new());
    played
        .f
        .artboard
        .build_semantic_tree(Some(manager.clone()), None);
    played.frames_until_settled();
    mount(&mut played, &["a", "b", "c"]);
    let runs: Vec<_> = ["a", "b", "c"]
        .map(|key| (in_row(&played, key, card::RUN), format!("Plan {key}")))
        .to_vec();
    write_once(&played, move |scene| {
        for (run, text) in &runs {
            assert_eq!(
                scene.set_text(run, TextValueRunBase::TEXT_PROPERTY_KEY, text),
                Ok(true)
            );
        }
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let texts = semantic_texts(&manager);
    let rows: Vec<_> = texts
        .iter()
        .filter(|text| text.starts_with("Plan"))
        .collect();
    assert_eq!(rows, ["Plan a", "Plan b", "Plan c"], "{texts:?}");
    write_once(&played, |scene| {
        assert_eq!(scene.move_row(&cards(), b"c", 0), Ok(true));
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let texts = semantic_texts(&manager);
    let rows: Vec<_> = texts
        .iter()
        .filter(|text| text.starts_with("Plan"))
        .collect();
    assert_eq!(rows, ["Plan c", "Plan a", "Plan b"], "{texts:?}");
}

/// A list with no `rowArtboardId`, whose rows come from list items as a
/// bound list hands them over: each row operation is refused as not keyed,
/// a path through it with a row key leads nowhere, and nothing changes.
#[test]
fn row_operations_on_a_list_that_is_not_keyed_are_refused() {
    let mut played = played(Cards::Unkeyed);
    let list = list(&played);
    let arena = list.retain_arena().expect("list arena");
    let items: Vec<_> = (0..2)
        .map(|_| arena.insert(ViewModelInstanceListItem::default()))
        .collect();
    ArtboardComponentList::update_list_occurrence(&list, &items);
    let source = played
        .f
        ._file
        .with_file(|file| file.artboard_at_source(1))
        .expect("row source");
    for index in 0..2 {
        let row = Artboard::nested_instance_from_handle(&source).expect("row instance");
        ArtboardComponentList::add_artboard_at_occurrence(&list, row, index, true);
    }
    played.frames_until_settled();
    let rows_before: Vec<_> = (0..2)
        .map(|index| {
            list.with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index))
                .flatten()
                .expect("a row")
        })
        .collect();
    let run = in_row(&played, "0", card::RUN);
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&played, move |scene| {
        log.borrow_mut().extend([
            scene.mount_row(&cards(), b"0"),
            scene.remove_row(&cards(), b"0"),
            scene.move_row(&cards(), b"0", 0),
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "x"),
        ]);
        assert_eq!(scene.changes(), 0);
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(
        *answers.borrow(),
        [
            Err(SceneWriteError::NotKeyedList),
            Err(SceneWriteError::NotKeyedList),
            Err(SceneWriteError::NotKeyedList),
            Err(SceneWriteError::PathNotFound),
        ]
    );
    assert_eq!(output.driver_rounds, 1);
    for (index, row) in rows_before.iter().enumerate() {
        let now = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index as i32))
            .flatten()
            .expect("a row");
        assert!(now.downgrade().ptr_eq(&row.downgrade()));
    }
    // A bound list still takes the list it is handed.
    ArtboardComponentList::update_list_occurrence(&list, &items[..1]);
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::artboard_count),
        Some(1)
    );
}

/// Each refusal changes nothing: a key not mounted, a key mounted twice, a
/// place past the end, a path through a keyed list with no key, a key named
/// for a nested artboard step, a key on a step that names no list, and a
/// keyed list whose record names an artboard the file does not have.
#[test]
fn each_row_refusal_changes_nothing() {
    let mut played = Played::new(import(&fixture::keyed_plans_fixture(Cards::Badged)));
    played.frames_until_settled();
    mount(&mut played, &["monthly", "yearly"]);
    played.frames_until_settled();
    let card_id = crate::host_semantics::source_artboard_global_id(&row_of(&played, "monthly"))
        .expect("the card's global id");
    let path = row_path(&played);
    let page_id = page_id(&played);
    let mut badge_path = path.clone();
    badge_path.push(PathStep {
        artboard_global_id: card_id,
        local_id: card::BADGE,
    });
    let no_key = SceneObject::nested(path.clone(), card::RUN);
    let not_mounted = in_row(&played, "weekly", card::RUN);
    let key_on_badge = SceneObject::in_rows(
        badge_path,
        vec![
            row_key(&played, "monthly"),
            RuntimeListRowKey {
                artboard_global_id: card_id,
                host_local_id: card::BADGE,
                key: b"monthly".as_slice().into(),
            },
        ],
        badge::TAG_COLOR,
    );
    let key_on_footer = SceneObject::in_rows(
        vec![PathStep {
            artboard_global_id: page_id,
            local_id: page::FOOTER,
        }],
        vec![RuntimeListRowKey {
            artboard_global_id: page_id,
            host_local_id: page::FOOTER,
            key: b"monthly".as_slice().into(),
        }],
        card::RUN,
    );
    let keys_left_over = SceneObject::in_rows(
        path,
        vec![row_key(&played, "monthly"), row_key(&played, "yearly")],
        card::RUN,
    );
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&played, move |scene| {
        let text = |scene: &mut SceneWriter<'_>, object: &SceneObject| {
            scene.set_text(object, TextValueRunBase::TEXT_PROPERTY_KEY, "x")
        };
        let answers = vec![
            scene.remove_row(&cards(), b"weekly"),
            scene.move_row(&cards(), b"weekly", 0),
            scene.mount_row(&cards(), b"monthly"),
            scene.move_row(&cards(), b"monthly", 2),
            scene.mount_row(&SceneObject::root(page::FOOTER), b"weekly"),
            text(scene, &no_key),
            text(scene, &not_mounted),
            scene.set_color(
                &key_on_badge,
                SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
                0xffaa_0000,
            ),
            text(scene, &key_on_footer),
            text(scene, &keys_left_over),
            scene.reset(&not_mounted),
        ];
        log.borrow_mut().extend(answers);
        assert_eq!(scene.changes(), 0);
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(
        *answers.borrow(),
        [
            Err(SceneWriteError::KeyNotFound),
            Err(SceneWriteError::KeyNotFound),
            Err(SceneWriteError::KeyExists),
            Err(SceneWriteError::PlaceOutOfRange),
            Err(SceneWriteError::NotKeyedList),
            Err(SceneWriteError::PathNotFound),
            Err(SceneWriteError::PathNotFound),
            Err(SceneWriteError::PathNotFound),
            Err(SceneWriteError::PathNotFound),
            Err(SceneWriteError::PathNotFound),
            Err(SceneWriteError::PathNotFound),
        ]
    );
    assert_eq!(output.driver_rounds, 1);
    assert_eq!(laid_out_keys(&played), ["monthly", "yearly"]);
    for key in ["monthly", "yearly"] {
        assert_eq!(run_text(&played, key), fixture::RUN_TEXT);
    }

    let mut missing = played_missing();
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&missing, move |scene| {
        log.borrow_mut().push(scene.mount_row(&cards(), b"monthly"));
        assert_eq!(scene.changes(), 0);
    });
    missing.step(FRAME, vec![]);
    assert_eq!(*answers.borrow(), [Err(SceneWriteError::NoRowArtboard)]);
    assert!(keys(&missing).is_empty());
}

fn played_missing() -> Played {
    played(Cards::MissingArtboard)
}

/// The badge artboard the row mounted by `key` hosts.
fn badge_of(played: &Played, key: &str) -> RuntimeArtboardInstanceHandle {
    row_object(played, key, card::BADGE)
        .with(|badge| {
            badge
                .as_nested_artboard()
                .and_then(|nested| nested.artboard_instance_default())
        })
        .flatten()
        .expect("the hosted badge")
}

fn tag_color(played: &Played, key: &str) -> u32 {
    let color = badge_of(played, key)
        .with_artboard(|badge| badge.objects().get(badge::TAG_COLOR).cloned().flatten())
        .expect("the tag's color");
    CoreRegistry::get_color_handle(&color, i32::from(SolidColorBase::COLOR_VALUE_PROPERTY_KEY))
        .expect("a color") as u32
}

/// A path of two steps, a row and then the badge its card hosts, writes the
/// badge of that row only.
#[test]
fn a_row_then_use_path_writes_one_use_of_one_row() {
    let mut played = Played::new(import(&fixture::keyed_plans_fixture(Cards::Badged)));
    played.frames_until_settled();
    mount(&mut played, &["monthly", "yearly"]);
    played.frames_until_settled();
    let card_id = crate::host_semantics::source_artboard_global_id(&row_of(&played, "yearly"))
        .expect("the card's global id");
    let mut path = row_path(&played);
    path.push(PathStep {
        artboard_global_id: card_id,
        local_id: card::BADGE,
    });
    let color = SceneObject::in_rows(path, vec![row_key(&played, "yearly")], badge::TAG_COLOR);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_color(
                &color,
                SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
                0xffaa_0000
            ),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    assert_eq!(tag_color(&played, "yearly"), 0xffaa_0000);
    assert_eq!(tag_color(&played, "monthly"), fixture::TAG_COLOR);
}

/// A driver that logs every report, and runs the write it is handed once, in
/// the first round of the next step.
type NextWrite = Rc<RefCell<Option<Box<dyn FnOnce(&mut SceneWriter<'_>)>>>>;

fn drive_logged(played: &Played) -> (Rc<RefCell<Vec<OwnedPlayerEvent>>>, NextWrite) {
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    let next: NextWrite = Rc::new(RefCell::new(None));
    let write = next.clone();
    played.drive(move |round, scene| {
        log.borrow_mut().extend(round.reports.iter().cloned());
        if round.index == 0
            && let Some(write) = write.borrow_mut().take()
        {
            write(scene);
        }
    });
    (heard, next)
}

/// A reset of an object inside a row drops the press the row's machine holds
/// there, as a reset inside a use does for the use's host machines: the
/// release after it reports nothing. A press held on another row, which the
/// reset did not reach, still reports its tap, with that row's key.
#[test]
fn a_reset_inside_a_row_drops_the_press_the_rows_machine_holds_there() {
    let mut played = played_with(&["a", "b"]);
    let (heard, next) = drive_logged(&played);
    let button = in_row(&played, "a", card::BUTTON);

    let at_a = button_center(&played, "a");
    played.step(FRAME, vec![pointer(PointerKind::Down, at_a)]);
    let reset = button.clone();
    *next.borrow_mut() = Some(Box::new(move |scene| {
        assert!(scene.reset(&reset).is_ok());
    }));
    played.step(FRAME, vec![]);
    played.step(FRAME, vec![pointer(PointerKind::Up, at_a)]);
    played.frames_until_settled();
    assert_eq!(names(&heard.borrow()), Vec::<String>::new());

    let at_b = button_center(&played, "b");
    played.step(FRAME, vec![pointer(PointerKind::Down, at_b)]);
    *next.borrow_mut() = Some(Box::new(move |scene| {
        assert!(scene.reset(&button).is_ok());
    }));
    played.step(FRAME, vec![]);
    played.step(FRAME, vec![pointer(PointerKind::Up, at_b)]);
    played.frames_until_settled();
    let heard = heard.borrow();
    assert_eq!(names(&heard), ["chosen"]);
    assert_eq!(
        heard[0].context.as_ref().expect("a context").rows(),
        [row_key(&played, "b")]
    );
}

/// A reset that names a keyed list's host rebuilds each row the list lays
/// out from the row artboard, with its machine at entry and the file's
/// values, under the same key and in the same place. Nothing is kept for
/// reuse: the old instances are gone. In the round of a remove, a mount and
/// a move, the round's order still holds after it. The rebuilt rows report
/// their taps with their keys.
#[test]
fn a_reset_of_a_keyed_list_rebuilds_its_rows_under_their_keys() {
    let mut played = played_with(&["a", "b", "c"]);
    let run = in_row(&played, "a", card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    assert_eq!(run_text(&played, "a"), "Chosen");
    let old_a = row_of(&played, "a").downgrade();
    let old_b = row_of(&played, "b").downgrade();
    let old_c = row_of(&played, "c").downgrade();

    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&played, move |scene| {
        log.borrow_mut().extend([
            scene.remove_row(&cards(), b"b"),
            scene.mount_row(&cards(), b"d"),
            scene.move_row(&cards(), b"c", 0),
            scene.reset(&cards()),
        ]);
    });
    played.step(FRAME, vec![]);
    assert_eq!(*answers.borrow(), [Ok(true); 4]);
    assert_eq!(keys(&played), ["c", "a", "d"]);
    assert_eq!(laid_out_keys(&played), ["c", "a", "d"]);
    for old in [&old_a, &old_b, &old_c] {
        assert!(old.upgrade().is_none(), "an old row is not kept");
    }
    assert_eq!(run_text(&played, "a"), fixture::RUN_TEXT);
    assert!(
        dot_x(&played, "a") < fixture::DOT_TRAVEL / 2.0,
        "the machine starts over: {}",
        dot_x(&played, "a")
    );

    let (heard, _) = drive_logged(&played);
    let at = button_center(&played, "a");
    played.tap(at.0, at.1);
    let heard = heard.borrow();
    assert_eq!(names(&heard), ["chosen"]);
    assert_eq!(
        heard[0].context.as_ref().expect("a context").rows(),
        [row_key(&played, "a")]
    );
}

/// Whether a list is keyed, and the artboard its rows are made from, are
/// what the file declared when the instance was made. A driver's write to
/// `rowArtboardId` is refused as an unknown property and changes nothing,
/// on a keyed list with rows mounted and on a list that is not keyed; a
/// write through the registry, the way an animation or a bind writes, leaves
/// it as it is. A keyed list also takes no list from a bind.
#[test]
fn a_lists_row_artboard_is_fixed_once_the_instance_is_made() {
    const ROW_ARTBOARD: u16 = ArtboardComponentList::ROW_ARTBOARD_ID_PROPERTY_KEY;
    let row_artboard = |list: &CoreHandle| {
        list.with_downcast::<ArtboardComponentList, _>(|list| {
            (list.is_keyed(), list.row_artboard_id())
        })
        .expect("the list")
    };

    let mut rows = played_with(&["a", "b"]);
    let keyed = list(&rows);
    let declared = row_artboard(&keyed);
    assert!(declared.0);
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&rows, move |scene| {
        log.borrow_mut().extend([
            scene.set_unsigned(&cards(), ROW_ARTBOARD, 0),
            scene.set_unsigned(&cards(), ROW_ARTBOARD, u32::MAX),
        ]);
        assert_eq!(scene.changes(), 0);
    });
    rows.step(FRAME, vec![]);
    assert_eq!(
        *answers.borrow(),
        [Err(SceneWriteError::UnknownProperty); 2]
    );
    CoreRegistry::set_uint_handle(&keyed, i32::from(ROW_ARTBOARD), u32::MAX);
    CoreRegistry::set_uint_handle(&keyed, i32::from(ROW_ARTBOARD), 0);
    assert_eq!(row_artboard(&keyed), declared);
    let arena = keyed.retain_arena().expect("list arena");
    let bound: Vec<_> = (0..3)
        .map(|_| arena.insert(ViewModelInstanceListItem::default()))
        .collect();
    ArtboardComponentList::update_list_occurrence(&keyed, &bound);
    rows.frames_until_settled();
    assert_eq!(keys(&rows), ["a", "b"]);
    assert_eq!(laid_out_keys(&rows), ["a", "b"]);

    let mut unkeyed = played(Cards::Unkeyed);
    let list = list(&unkeyed);
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    write_once(&unkeyed, move |scene| {
        log.borrow_mut().extend([
            scene.set_unsigned(&cards(), ROW_ARTBOARD, 1),
            scene.mount_row(&cards(), b"a"),
        ]);
        assert_eq!(scene.changes(), 0);
    });
    unkeyed.step(FRAME, vec![]);
    assert_eq!(
        *answers.borrow(),
        [
            Err(SceneWriteError::UnknownProperty),
            Err(SceneWriteError::NotKeyedList)
        ]
    );
    CoreRegistry::set_uint_handle(&list, i32::from(ROW_ARTBOARD), 1);
    assert_eq!(row_artboard(&list), (false, u32::MAX));
}
