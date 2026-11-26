//! A keyed list marked as a page's rows (`isPageRows`): a driver page. The
//! plans screen's "Cards" list is keyed, declares the card and the banner,
//! and carries the flag (`Cards::PageRows`), so the record-free root is a
//! page whose rows a driver mounts and a host puts to sleep by key.
//!
//! Oracle: the CSS block flow, where a sleeping row takes no room and the
//! rows after it close up; the draw and hit streams of the same page with
//! the row absent; the row's own state from before it slept.
use super::*;
use crate::hosted_scene;
use crate::page_player::{Error, ErrorKind};

/// The card's text draws (and the heading's): the file paints every text
/// 0xff112233.
const TEXT_COLOR: &str = "color=0xff112233";

/// The driver page with no rows, settled.
fn driver_page() -> Played {
    played(Cards::PageRows)
}

/// The driver page with a row mounted from the artboard beside each key,
/// settled.
fn driver_page_with(rows: &[(&'static str, u32)]) -> Played {
    let mut played = driver_page();
    mount_from(&mut played, rows);
    played.frames_until_settled();
    played
}

const CARD: u32 = fixture::CARD_ARTBOARD;
const BANNER: u32 = fixture::BANNER_ARTBOARD;

/// Put the row mounted by `key` on the "Cards" list to sleep, or wake it,
/// between steps.
fn sleep(played: &mut Played, key: &str, collapsed: bool) -> Result<(), Error> {
    played.player.set_keyed_row_collapsed(
        &mut played.board,
        "Cards",
        key.as_bytes(),
        collapsed,
        &[],
        None,
    )
}

/// As `sleep`, naming the row by its place in the list.
fn sleep_at(played: &mut Played, index: i32, collapsed: bool) -> Result<(), Error> {
    played
        .player
        .set_row_collapsed(&mut played.board, "Cards", index, collapsed, &[], None)
}

fn refused<T: std::fmt::Debug>(result: Result<T, Error>) -> ErrorKind {
    result.expect_err("refused").kind
}

/// Whether the row mounted by `key` sleeps.
fn asleep(played: &Played, key: &str) -> bool {
    list(played)
        .with_downcast::<ArtboardComponentList, _>(|list| {
            let index = list
                .keyed_row_index(key.as_bytes())
                .expect("a laid out row");
            list.live_move_row_collapsed(index)
        })
        .expect("the list")
}

/// The keys of the rows the list lays out and draws, in order: the awake
/// ones.
fn awake_keys(played: &Played) -> Vec<String> {
    list(played)
        .with_downcast_mut::<ArtboardComponentList, _>(|list| {
            (0..list.artboard_count() as i32)
                .filter(|index| list.layout_node(*index).is_some())
                .map(|index| {
                    let item = list.list_item(index).expect("an item");
                    String::from_utf8_lossy(list.key_of_row(&item).expect("a keyed row"))
                        .into_owned()
                })
                .collect()
        })
        .expect("the list")
}

fn text_draws(played: &Played) -> usize {
    drawn_paths(&played.f)
        .iter()
        .filter(|line| line.contains(TEXT_COLOR))
        .count()
}

fn strip_draws(played: &Played) -> usize {
    let strip = format!("color=0x{:08x}", fixture::BANNER_COLOR);
    drawn_paths(&played.f)
        .iter()
        .filter(|line| line.contains(&strip))
        .count()
}

/// The names of the events a tap at `at` reports.
fn tapped(played: &mut Played, at: (f32, f32)) -> Vec<String> {
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    played.drive(move |round, _| log.borrow_mut().extend(names(round.reports)));
    played.tap(at.0, at.1);
    played.player.set_driver(None).expect("no step is running");
    heard.take()
}

fn content_height(played: &Played) -> Result<f32, Error> {
    played.player.content_height(&played.board)
}

/// Proof 1. A record-free root whose own artboard holds a keyed list with
/// `isPageRows` is a page: its rows sleep by key, and deciding so scans the
/// file's capability once across all its frames. A keyed list without the
/// flag, and the flag on a list that is not keyed, leave the root an
/// ordinary artboard: sleep is refused, nothing is kept for it, and the
/// capability is scanned once too.
#[test]
fn a_flagged_keyed_list_makes_a_record_free_root_a_page() {
    hosted_scene::measure_walks(true);
    let mut page = driver_page_with(&[("card", CARD), ("banner", BANNER)]);
    for _ in 0..8 {
        page.step(FRAME, vec![]);
    }
    let walks = hosted_scene::measure_walks(false);
    assert_eq!(walks.capability_scan, 1, "{walks:?}");
    let root = page.board.native_handle();
    assert!(hosted_scene::enabled(&root));
    sleep(&mut page, "card", true).expect("a driver page sleeps a row by key");
    assert!(asleep(&page, "card"));

    for cards in [Cards::WithBanner, Cards::PageRowsUnkeyed] {
        let before = hosted_scene::bookkeeping_counts();
        hosted_scene::measure_walks(true);
        let mut plain = played(cards);
        if cards == Cards::WithBanner {
            mount_from(&mut plain, &[("card", CARD), ("banner", BANNER)]);
            plain.frames_until_settled();
        }
        let walks = hosted_scene::measure_walks(false);
        assert_eq!(walks.capability_scan, 1, "{cards:?}: {walks:?}");
        let root = plain.board.native_handle();
        assert!(!hosted_scene::enabled(&root), "{cards:?}");
        assert_eq!(
            refused(sleep(&mut plain, "card", true)),
            ErrorKind::InvalidArgument,
            "{cards:?}"
        );
        assert_eq!(
            refused(sleep_at(&mut plain, 0, true)),
            ErrorKind::InvalidArgument,
            "{cards:?}"
        );
        assert_eq!(
            refused(content_height(&plain)),
            ErrorKind::InvalidArgument,
            "{cards:?}"
        );
        assert_eq!(hosted_scene::bookkeeping_counts(), before, "{cards:?}");
        drop(plain);
    }
}

/// Proof 2. A card mounted above a banner sleeps by its key while its
/// machine is still playing "intro": the banner and the footer close up
/// under the heading, the card's text is not drawn, a tap where its button
/// was reports nothing, and its dot holds still over many frames. Woken by
/// its key it is back above the banner where it was, the same instance with
/// the scroll written into it, and its machine plays on to the end.
#[test]
fn a_sleeping_row_is_not_laid_out_drawn_hit_or_advanced_and_wakes_with_its_state() {
    let mut played = driver_page();
    mount_from(&mut played, &[("card", CARD), ("banner", BANNER)]);
    let scroll = in_row(&played, "card", card::SCROLL);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_number(&scroll, fixture::SCROLL_OFFSET_Y, -50.0),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    let dot = dot_x(&played, "card");
    assert!(dot < fixture::DOT_TRAVEL / 2.0, "intro is playing: {dot}");
    let (top, height) = row_frame(&played, "card");
    assert!(near(top, fixture::HEADING_HEIGHT), "{top}");
    let banner_top = row_frame(&played, "banner").0;
    assert!(near(banner_top, fixture::HEADING_HEIGHT + height));
    let footer = footer_top(&played);
    let button = button_center(&played, "card");
    let instance = row_of(&played, "card");
    let texts = text_draws(&played);
    assert_eq!(texts, 2, "the title and the card's words");

    sleep(&mut played, "card", true).expect("the card sleeps");
    played.step(FRAME, vec![]);
    assert!(asleep(&played, "card"));
    assert_eq!(keys(&played), ["card", "banner"]);
    assert_eq!(awake_keys(&played), ["banner"]);
    assert!(near(
        row_frame(&played, "banner").0,
        fixture::HEADING_HEIGHT
    ));
    assert!(near(
        footer_top(&played),
        fixture::HEADING_HEIGHT + fixture::BANNER_HEIGHT
    ));
    assert_eq!(text_draws(&played), 1, "only the title is drawn");
    assert_eq!(strip_draws(&played), 1);
    assert_eq!(tapped(&mut played, button), Vec::<String>::new());
    for _ in 0..60 {
        played.step(FRAME, vec![]);
    }
    assert_eq!(dot_x(&played, "card"), dot, "the card did not advance");

    sleep(&mut played, "card", false).expect("the card wakes");
    played.step(FRAME, vec![]);
    assert!(!asleep(&played, "card"));
    assert_eq!(awake_keys(&played), ["card", "banner"]);
    assert!(
        row_of(&played, "card")
            .downgrade()
            .ptr_eq(&instance.downgrade())
    );
    assert!(near(row_frame(&played, "card").0, top));
    assert!(near(row_frame(&played, "banner").0, banner_top));
    assert!(near(footer_top(&played), footer));
    assert_eq!(text_draws(&played), texts);
    assert_eq!(scroll_y(&played, "card"), -50.0);
    played.frames_until_settled();
    assert!(near(dot_x(&played, "card"), fixture::DOT_TRAVEL));
    assert_eq!(tapped(&mut played, button), ["chosen"]);
}

/// Proof 3. A driver writes long words into a sleeping card's label: the
/// write lands in the row (its run holds the words), and nothing on the page
/// moves or draws differently. Woken, the label wraps, the card grows, and
/// the banner below it moves down by as much, in the frame after the wake.
#[test]
fn a_write_to_a_sleeping_row_is_laid_out_and_drawn_when_it_wakes() {
    let mut played = driver_page_with(&[("card", CARD), ("banner", BANNER)]);
    let label = card_box(&played, "card", card::LABEL)[3];
    let (_, height) = row_frame(&played, "card");
    let banner_awake = row_frame(&played, "banner").0;
    sleep(&mut played, "card", true).expect("the card sleeps");
    played.frames_until_settled();
    let banner_asleep = row_frame(&played, "banner").0;
    let footer = footer_top(&played);
    let drawn = drawn_paths(&played.f);

    let run = in_row(&played, "card", card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, LONG),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    assert_eq!(run_text(&played, "card"), LONG);
    assert!(near(row_frame(&played, "banner").0, banner_asleep));
    assert!(near(footer_top(&played), footer));
    assert_eq!(drawn_paths(&played.f), drawn, "nothing draws differently");

    sleep(&mut played, "card", false).expect("the card wakes");
    played.step(FRAME, vec![]);
    let grown = card_box(&played, "card", card::LABEL)[3] - label;
    assert!(grown > 10.0, "the label wraps: {grown}");
    assert!(near(row_frame(&played, "card").1, height + grown));
    assert!(near(row_frame(&played, "banner").0, banner_awake + grown));
    assert_eq!(text_draws(&played), 2);
    assert!(
        drawn_paths(&played.f).len() > drawn.len(),
        "the card's longer words are drawn"
    );
}

/// Proof 4. The content height is the intrinsic height of the top row, the
/// last awake row in the list's order: the banner under a card, the card
/// once a move puts it last, the banner again while the card sleeps, and
/// NotFound once every row sleeps.
#[test]
fn content_height_reads_the_top_awake_row() {
    let mut played = driver_page_with(&[("card", CARD), ("banner", BANNER)]);
    let card_height = row_frame(&played, "card").1;
    let height = content_height(&played).expect("a height");
    assert!(near(height, fixture::BANNER_HEIGHT), "{height}");

    write_once(&played, |scene| {
        assert_eq!(scene.move_row(&cards(), b"card", 1), Ok(true));
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let height = content_height(&played).expect("a height");
    assert!(near(height, card_height), "{height} and {card_height}");

    sleep(&mut played, "card", true).expect("the card sleeps");
    played.step(FRAME, vec![]);
    let height = content_height(&played).expect("a height");
    assert!(near(height, fixture::BANNER_HEIGHT), "{height}");

    sleep(&mut played, "banner", true).expect("the banner sleeps");
    played.step(FRAME, vec![]);
    assert_eq!(refused(content_height(&played)), ErrorKind::NotFound);
}

/// Proof 5. A sleeping row moved by the driver stays asleep at its new
/// place; removed, its sleep goes with it, and a row mounted again under its
/// key is awake. No frame of the driver page, a round of row operations
/// among them, runs the screens list's membership flush.
#[test]
fn sleep_follows_a_moved_row_and_is_dropped_by_a_remove() {
    let mut played = driver_page_with(&[("a", CARD), ("b", BANNER), ("c", CARD)]);
    sleep(&mut played, "a", true).expect("a sleeps");
    hosted_scene::measure_walks(true);
    write_once(&played, |scene| {
        assert_eq!(scene.move_row(&cards(), b"a", 2), Ok(true));
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    assert_eq!(keys(&played), ["b", "c", "a"]);
    assert!(asleep(&played, "a"));
    assert_eq!(awake_keys(&played), ["b", "c"]);
    assert!(near(row_frame(&played, "b").0, fixture::HEADING_HEIGHT));

    write_once(&played, |scene| {
        assert_eq!(scene.remove_row(&cards(), b"a"), Ok(true));
    });
    played.step(FRAME, vec![]);
    write_once(&played, |scene| {
        assert_eq!(scene.mount_row(&cards(), b"a"), Ok(true));
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let walks = hosted_scene::measure_walks(false);
    assert_eq!(walks.membership_flush, 0, "{walks:?}");
    assert_eq!(keys(&played), ["b", "c", "a"]);
    assert!(!asleep(&played, "a"));
    assert_eq!(awake_keys(&played), ["b", "c", "a"]);
    let has_sleeping = list(&played)
        .with_downcast::<ArtboardComponentList, _>(
            ArtboardComponentList::live_move_has_sleeping_rows,
        )
        .expect("the list");
    assert!(!has_sleeping);
}

/// Proof 6. Two rows of one artboard on a driver page: either sleeps and
/// wakes by its key, and the content height reads the top one awake. A step
/// is refused on a driver page and leaves the page playing.
#[test]
fn two_rows_of_one_artboard_sleep_and_steps_are_refused() {
    let mut played = driver_page_with(&[("monthly", CARD), ("yearly", CARD)]);
    let card_height = row_frame(&played, "yearly").1;
    assert!(near(
        content_height(&played).expect("a height"),
        card_height
    ));
    sleep(&mut played, "yearly", true).expect("yearly sleeps");
    played.step(FRAME, vec![]);
    assert_eq!(awake_keys(&played), ["monthly"]);
    assert!(near(
        content_height(&played).expect("a height"),
        card_height
    ));
    sleep(&mut played, "monthly", true).expect("monthly sleeps");
    played.step(FRAME, vec![]);
    assert_eq!(awake_keys(&played), Vec::<String>::new());
    sleep(&mut played, "yearly", false).expect("yearly wakes");
    played.step(FRAME, vec![]);
    assert_eq!(awake_keys(&played), ["yearly"]);
    assert!(near(
        row_frame(&played, "yearly").0,
        fixture::HEADING_HEIGHT
    ));

    let begin = played
        .player
        .step_begin(&mut played.board, "Plan card", None, &[], None);
    assert_eq!(refused(begin), ErrorKind::InvalidArgument);
    let end = played
        .player
        .step_end(&mut played.board, "Plan card", None, &[], None);
    assert_eq!(refused(end), ErrorKind::InvalidArgument);
    assert!(!crate::page_player::is_poisoned(
        &played.board.native_handle()
    ));
    played.step(FRAME, vec![]);
    sleep(&mut played, "monthly", false).expect("monthly wakes");
    played.step(FRAME, vec![]);
    assert_eq!(awake_keys(&played), ["monthly", "yearly"]);
}

/// Proof 7. A reset that names the page's list rebuilds each row and keeps a
/// sleeping one asleep: still not laid out or drawn, and it wakes as a
/// fresh row with the file's values.
#[test]
fn a_reset_of_the_page_list_keeps_a_sleeping_row_asleep() {
    let mut played = driver_page_with(&[("card", CARD), ("banner", BANNER)]);
    let run = in_row(&played, "card", card::RUN);
    write_once(&played, move |scene| {
        assert_eq!(
            scene.set_text(&run, TextValueRunBase::TEXT_PROPERTY_KEY, "Chosen"),
            Ok(true)
        );
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    let (top, _) = row_frame(&played, "card");
    let old = row_of(&played, "card");
    sleep(&mut played, "card", true).expect("the card sleeps");
    played.frames_until_settled();
    write_once(&played, |scene| {
        assert_eq!(scene.reset(&cards()), Ok(true));
    });
    played.step(FRAME, vec![]);
    played.frames_until_settled();
    assert!(!row_of(&played, "card").downgrade().ptr_eq(&old.downgrade()));
    assert!(asleep(&played, "card"));
    assert_eq!(awake_keys(&played), ["banner"]);
    assert!(near(
        row_frame(&played, "banner").0,
        fixture::HEADING_HEIGHT
    ));
    assert_eq!(text_draws(&played), 1, "only the title is drawn");

    sleep(&mut played, "card", false).expect("the card wakes");
    played.step(FRAME, vec![]);
    assert_eq!(awake_keys(&played), ["card", "banner"]);
    assert!(near(row_frame(&played, "card").0, top));
    assert_eq!(run_text(&played, "card"), fixture::RUN_TEXT);
    assert_eq!(text_draws(&played), 2);
}

/// Proof 8. The index call and the key call name the same row on a driver
/// page: a row put to sleep by its place wakes by its key, and one put to
/// sleep by its key wakes by its place. The key call refuses a key not
/// mounted, a list the page does not have and a list that is not the
/// page's, and changes nothing.
#[test]
fn index_and_key_calls_agree_and_refusals_change_nothing() {
    let mut played = driver_page_with(&[("a", CARD), ("b", BANNER), ("c", CARD)]);
    sleep_at(&mut played, 1, true).expect("b sleeps by its place");
    assert!(asleep(&played, "b"));
    sleep(&mut played, "b", false).expect("b wakes by its key");
    assert!(!asleep(&played, "b"));
    sleep(&mut played, "c", true).expect("c sleeps by its key");
    assert!(asleep(&played, "c"));
    sleep_at(&mut played, 2, false).expect("c wakes by its place");
    assert!(!asleep(&played, "c"));

    let revision = hosted_scene::membership_revision(&played.board.native_handle().core_handle());
    assert_eq!(
        refused(sleep(&mut played, "d", true)),
        ErrorKind::InvalidArgument
    );
    let elsewhere =
        played
            .player
            .set_keyed_row_collapsed(&mut played.board, "Heading", b"a", true, &[], None);
    assert_eq!(refused(elsewhere), ErrorKind::InvalidArgument);
    let nowhere =
        played
            .player
            .set_keyed_row_collapsed(&mut played.board, "Nowhere", b"a", true, &[], None);
    assert_eq!(refused(nowhere), ErrorKind::InvalidArgument);
    assert_eq!(
        hosted_scene::membership_revision(&played.board.native_handle().core_handle()),
        revision
    );
    assert!(!crate::page_player::is_poisoned(
        &played.board.native_handle()
    ));
    assert_eq!(awake_keys(&played), ["a", "b", "c"]);
}

/// `isPageRows` is what the file declared: a driver's write to it is refused
/// as an unknown property, and a write through the registry leaves it as it
/// is.
#[test]
fn a_lists_page_rows_flag_is_fixed_once_the_instance_is_made() {
    const PAGE_ROWS: u16 = ArtboardComponentList::IS_PAGE_ROWS_PROPERTY_KEY;
    let played = driver_page();
    let flagged = |played: &Played| {
        list(played)
            .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::is_page_rows)
            .expect("the list")
    };
    assert!(flagged(&played));
    let answers = Rc::new(RefCell::new(Vec::new()));
    let log = answers.clone();
    let mut played = played;
    write_once(&played, move |scene| {
        log.borrow_mut().extend([
            scene.set_bool(&cards(), PAGE_ROWS, false),
            scene.set_unsigned(&cards(), PAGE_ROWS, 0),
        ]);
        assert_eq!(scene.changes(), 0);
    });
    played.step(FRAME, vec![]);
    assert_eq!(
        *answers.borrow(),
        [Err(SceneWriteError::UnknownProperty); 2]
    );
    CoreRegistry::set_bool_handle(&list(&played), i32::from(PAGE_ROWS), false);
    assert!(flagged(&played));
}
