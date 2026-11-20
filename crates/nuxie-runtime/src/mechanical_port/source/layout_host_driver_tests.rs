//! A host driving a player's artboard from outside the file
//! (`page_player::HostDriver`): files with no view model and no binds,
//! whose listeners only report events, and a driver that answers them with
//! property writes inside the same step.
//!
//! Oracle: CSS arithmetic for where boxes land, and the same scene changed
//! through the property setters directly and played over ordinary frames
//! for where text puts them and what a draw shows.
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::page_player::{
    DriverFault, DriverRound, HostDriver, MAX_DRIVER_ROUNDS, OwnedPlayerEvent, PagePlayer,
    Playback, Pointer, PointerKind, SceneObject, SceneWriteError, SceneWriter, Step,
};
use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
use crate::source::generated::text::text_value_run_base::TextValueRunBase;
use crate::{ArtboardInstance, page_player::Output};

const ALIGN_ITEMS: u16 = 61013;
const SCROLL_OFFSET_Y: u16 = 760;

/// A driver from a closure, which sees every round.
struct FnDriver<D>(D);

impl<D: FnMut(&DriverRound<'_>, &mut SceneWriter<'_>)> HostDriver for FnDriver<D> {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault> {
        (self.0)(&round, scene);
        Ok(())
    }
}

/// A scene played by a page player on its one state machine.
struct Played {
    f: Scene,
    board: ArtboardInstance,
    playback: Playback,
    player: PagePlayer,
}

impl Played {
    fn new(f: Scene) -> Self {
        let mut board =
            ArtboardInstance::from_native_handle(f._file.clone(), 0, f.artboard.clone());
        let machine = board.state_machine_instance(0).expect("the state machine");
        let mut played = Self {
            f,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            player: PagePlayer::default(),
        };
        played.step(0.0, vec![]);
        played
    }

    fn drive(&self, driver: impl FnMut(&DriverRound<'_>, &mut SceneWriter<'_>) + 'static) {
        self.player
            .set_driver(Some(Box::new(FnDriver(driver))))
            .expect("no step is running");
    }

    fn step(&mut self, elapsed_seconds: f32, pointers: Vec<Pointer>) -> Output {
        self.player
            .step(
                &mut self.board,
                &mut self.playback,
                Step {
                    pointers,
                    elapsed_seconds,
                    ..Step::default()
                },
                &[],
                None,
            )
            .expect("the step")
    }

    /// A press and release at a point, in one frame.
    fn tap(&mut self, x: f32, y: f32) -> Output {
        let pointer = |kind| Pointer {
            kind,
            x,
            y,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        };
        self.step(
            FRAME,
            vec![pointer(PointerKind::Down), pointer(PointerKind::Up)],
        )
    }

    /// Frames until the player says it is settled; the last frame's output.
    fn settle(&mut self) -> Output {
        for _ in 0..30 {
            let output = self.step(FRAME, vec![]);
            if output.settled {
                return output;
            }
        }
        panic!("the player never settled");
    }

    fn local_id(&self, handle: &CoreHandle) -> usize {
        usize::try_from(self.f.artboard.with_artboard(|a| a.object_index(handle)))
            .expect("an object of the artboard")
    }

    fn object(&self, name: &str) -> SceneObject {
        SceneObject::root(self.local_id(&self.f.object(name)))
    }
}

fn names(events: &[OwnedPlayerEvent]) -> Vec<String> {
    events
        .iter()
        .map(|event| String::from_utf8_lossy(event.name.as_deref().unwrap_or_default()).into())
        .collect()
}

/// The run holding `text`.
fn run(f: &Scene, text: &str) -> CoreHandle {
    f.artboard
        .with_artboard(|a| a.objects().to_vec())
        .into_iter()
        .flatten()
        .find(|handle| {
            handle.is_type_of(TextValueRunBase::TYPE_KEY)
                && CoreRegistry::get_string_handle(
                    handle,
                    i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
                )
                .as_deref()
                    == Some(text)
        })
        .unwrap_or_else(|| panic!("a run of {text:?}"))
}

/// A flex row: a label sized by its words, then a 50 point box painted
/// 0xff112233.
fn labelled_row() -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(DISPLAY, 1), U(ALIGN_ITEMS, 2)].as_slice(),
                &width(400.0),
                &height(300.0),
            ]
            .concat(),
            vec![
                B {
                    link: Some("tap"),
                    ..css("Label", [], vec![])
                },
                B {
                    paints: vec![Paint::Fill(0xff11_2233)],
                    ..css(
                        "After",
                        [
                            [F(FLEX_SHRINK, 0.0)].as_slice(),
                            &width(50.0),
                            &height(40.0),
                        ]
                        .concat(),
                        vec![],
                    )
                },
            ],
        ),
    )
}

fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 0.05
}

/// The draw's path lines, without the factory's own handle numbers.
fn drawn_paths(f: &Scene) -> Vec<String> {
    f.draw_lines()
        .into_iter()
        .filter(|line| line.starts_with("drawPath"))
        .collect()
}

/// A tap's report reaches the driver in the step that took the tap; the
/// driver's text write is laid out in that same step, and the one draw
/// after it shows the new words, as the setter written directly and played
/// over ordinary frames shows them.
#[test]
fn a_driver_text_write_lays_out_and_draws_in_the_tapped_step() {
    let oracle = labelled_row();
    let machine = oracle.link_machine();
    CoreRegistry::set_string_handle(
        &run(&oracle, PLAIN_WORDS),
        i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
        String::new(),
    );
    oracle.advance(3);
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    let link_only = oracle.bounds("After")[0];

    let mut played = Played::new(labelled_row());
    played.settle();
    let both = played.f.bounds("After")[0];
    assert!(link_only < both - 30.0, "{link_only} and {both}");
    let plain = played.local_id(&run(&played.f, PLAIN_WORDS));
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    played.drive(move |round, scene| {
        log.borrow_mut().extend(names(round.reports));
        if names(round.reports).iter().any(|name| name == "tap") {
            assert_eq!(
                scene.set_text(
                    &SceneObject::root(plain),
                    TextValueRunBase::TEXT_PROPERTY_KEY,
                    "",
                ),
                Ok(true)
            );
        }
    });
    let output = played.tap(30.0, 10.0);
    assert_eq!(*heard.borrow(), ["tap"]);
    assert_eq!(names(&output.events), ["tap"]);
    assert!(output.dirty);
    assert_eq!(output.driver_rounds, 2);
    assert!(
        near(played.f.bounds("After")[0], link_only),
        "laid out in the tapped step: {:?}, not {link_only}",
        played.f.bounds("After")
    );
    assert_eq!(drawn_paths(&played.f), drawn_paths(&oracle));
}

/// A column: a button that hears a tap, a box below it, and a box below that.
fn hide_column() -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [width(400.0), height(300.0)].concat(),
            vec![
                B {
                    listener: Some("tap"),
                    opaque: true,
                    ..css("Button", [width(100.0), height(40.0)].concat(), vec![])
                },
                B {
                    listener: Some("hidden tapped"),
                    opaque: true,
                    paints: vec![Paint::Fill(0xff11_2233)],
                    ..css("Hidden", [width(100.0), height(40.0)].concat(), vec![])
                },
                css("Below", [width(100.0), height(40.0)].concat(), vec![]),
            ],
        ),
    )
}

/// `display: none` written by a driver hides a box in place in the step
/// that asked for it: what is below moves up, and the box hears no tap. The
/// display written back shows it again in its step.
#[test]
fn a_driver_display_write_hides_and_shows_in_the_same_step() {
    let mut played = Played::new(hide_column());
    played.settle();
    assert_eq!(played.f.bounds("Below")[1], 80.0);
    let record = played.f.record("Hidden");
    let display = SceneObject::root(played.local_id(&record));
    let shown = CoreRegistry::get_uint_handle(&record, i32::from(DISPLAY)).expect("a display");
    let hidden = Rc::new(RefCell::new(false));
    let state = hidden.clone();
    played.drive(move |round, scene| {
        if names(round.reports).iter().any(|name| name == "tap") {
            let hide = !*state.borrow();
            *state.borrow_mut() = hide;
            let value = if hide { NONE as u32 } else { shown };
            assert_eq!(scene.set_unsigned(&display, DISPLAY, value), Ok(true));
        }
    });

    played.tap(50.0, 20.0);
    assert_eq!(played.f.bounds("Below")[1], 40.0, "moved up in the step");
    assert!(!tapped(&played.f, "Hidden", (50.0, 60.0)));
    let output = played.tap(50.0, 60.0);
    assert!(
        names(&output.events).is_empty(),
        "{:?}",
        names(&output.events)
    );

    played.tap(50.0, 20.0);
    assert_eq!(played.f.bounds("Below")[1], 80.0, "moved down in the step");
    assert!(tapped(&played.f, "Hidden", (50.0, 60.0)));
    let output = played.tap(50.0, 60.0);
    assert_eq!(names(&output.events), ["hidden tapped"]);
}

/// Reset returns a scroll box under the object to its authored offset and
/// leaves alone a color a host wrote there.
#[test]
fn reset_returns_a_scroll_offset_and_keeps_a_host_written_color() {
    let mut played = Played::new(scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Fill(0xff11_2233)],
            ..css_scroll_box(Some("tap"), css("Item", [], vec![]))
        },
    ));
    played.settle();
    let color = played.f.paint_child("Viewport", SolidColorBase::TYPE_KEY);
    let color_id = SceneObject::root(played.local_id(&color));
    let viewport = played.object("Viewport");
    let asked = Rc::new(RefCell::new(0usize));
    let asks = asked.clone();
    played.drive(move |round, scene| {
        if round.index != 0 || !names(round.reports).iter().any(|name| name == "tap") {
            return;
        }
        *asks.borrow_mut() += 1;
        if *asks.borrow() == 1 {
            assert_eq!(
                scene.set_color(
                    &color_id,
                    SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
                    0xff44_5566
                ),
                Ok(true)
            );
        } else {
            assert_eq!(scene.reset(&viewport), Ok(true));
        }
    });
    played.tap(100.0, 20.0);
    let scroll = played
        .f
        .artboard
        .with_artboard(|a| a.find_all_handles::<ScrollConstraint>())
        .remove(0);
    assert!(CoreRegistry::set_double_handle(
        &scroll,
        i32::from(SCROLL_OFFSET_Y),
        -50.0
    ));
    played.settle();
    assert_eq!(played.f.scroll_offset("List"), (0.0, -50.0));

    played.tap(100.0, 20.0);
    assert_eq!(*asked.borrow(), 2);
    assert_eq!(played.f.scroll_offset("List"), (0.0, 0.0));
    assert_eq!(
        CoreRegistry::get_color_handle(&color, i32::from(SolidColorBase::COLOR_VALUE_PROPERTY_KEY)),
        Some(0xff44_5566_u32 as i32)
    );
}

/// A tap's report names the listener's target object, by local id in the
/// artboard its path leads to: the root's own box with no path, and a nested
/// artboard's box with a path through the nested artboard.
#[test]
fn a_tap_report_carries_its_listener_target_and_path() {
    let mut played = Played::new(scene(
        (400.0, 300.0),
        css(
            "Page",
            [width(400.0), height(300.0)].concat(),
            vec![
                B {
                    listener: Some("tap"),
                    opaque: true,
                    ..css("Button", [width(100.0), height(40.0)].concat(), vec![])
                },
                nested_button(true, &[(ListenerType::Click, "nested tap")]),
            ],
        ),
    ));
    played.settle();
    let reports = Rc::new(RefCell::new(Vec::new()));
    let log = reports.clone();
    played.drive(move |round, _| log.borrow_mut().extend(round.reports.iter().cloned()));

    played.tap(50.0, 20.0);
    let button = played.local_id(&played.f.object("Button"));
    {
        let reports = reports.borrow();
        assert_eq!(names(&reports), ["tap"]);
        assert_eq!(reports[0].listener_target_local_id, Some(button));
        assert!(
            reports[0]
                .context
                .as_ref()
                .is_none_or(|c| c.path().is_empty())
        );
    }

    reports.borrow_mut().clear();
    played.tap(50.0, 60.0);
    let nested = played.local_id(&played.f.object("Nested"));
    let reports = reports.borrow();
    assert_eq!(names(&reports), ["nested tap"]);
    // The nested artboard: 0 the artboard, 1 its button.
    assert_eq!(reports[0].listener_target_local_id, Some(1));
    let path = reports[0].context.as_ref().expect("a path").path();
    assert_eq!(path.len(), 1, "{path:?}");
    assert_eq!(path[0].local_id, nested);
}

/// A driver that writes in every round is cut off at the bound: the step
/// returns unsettled after `MAX_DRIVER_ROUNDS` rounds, with every write laid
/// out, and the next step asks the driver again.
#[test]
fn a_driver_writing_every_round_is_cut_off_at_the_bound() {
    let mut played = Played::new(hide_column());
    played.settle();
    let color = played.f.paint_child("Hidden", SolidColorBase::TYPE_KEY);
    let color_id = SceneObject::root(played.local_id(&color));
    let rounds = Rc::new(RefCell::new(Vec::new()));
    let log = rounds.clone();
    played.drive(move |round, scene| {
        log.borrow_mut().push(round.index);
        let value = 0xff00_0000 | u32::try_from(log.borrow().len()).expect("a count");
        assert_eq!(
            scene.set_color(&color_id, SolidColorBase::COLOR_VALUE_PROPERTY_KEY, value),
            Ok(true)
        );
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, MAX_DRIVER_ROUNDS);
    assert_eq!(*rounds.borrow(), (0..MAX_DRIVER_ROUNDS).collect::<Vec<_>>());
    assert!(!output.settled);
    assert_eq!(
        CoreRegistry::get_color_handle(&color, i32::from(SolidColorBase::COLOR_VALUE_PROPERTY_KEY)),
        Some((0xff00_0000_u32 | MAX_DRIVER_ROUNDS as u32) as i32)
    );
    let output = played.step(FRAME, vec![]);
    assert_eq!(output.driver_rounds, MAX_DRIVER_ROUNDS);
    assert_eq!(rounds.borrow().len(), 2 * MAX_DRIVER_ROUNDS);
}

/// A still artboard whose machine only reports goes quiet: settled, with
/// nothing to draw. A driver's write wakes it for that step alone; the next
/// frame is settled again and draws nothing new.
#[test]
fn a_still_reporting_artboard_goes_quiet_after_a_write() {
    let mut played = Played::new(hide_column());
    let quiet = played.settle();
    assert!(!quiet.keep_going && !quiet.dirty, "quiet before any write");
    let color = played.f.paint_child("Hidden", SolidColorBase::TYPE_KEY);
    let color_id = SceneObject::root(played.local_id(&color));
    played.drive(move |round, scene| {
        if names(round.reports).iter().any(|name| name == "tap") {
            assert_eq!(
                scene.set_color(
                    &color_id,
                    SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
                    0xff44_5566
                ),
                Ok(true)
            );
        }
    });
    let written = played.tap(50.0, 20.0);
    assert!(written.dirty, "the write asks for a draw");
    assert_eq!(written.driver_rounds, 2);
    for frame in 0..3 {
        let output = played.step(FRAME, vec![]);
        assert!(output.settled, "frame {frame} after the write");
        assert!(!output.keep_going, "frame {frame} after the write");
        assert!(!output.dirty, "frame {frame} after the write");
        assert_eq!(output.driver_rounds, 1);
    }
}

/// A refused write changes nothing and says why.
#[test]
fn a_refused_write_names_its_reason() {
    let mut played = Played::new(hide_column());
    played.settle();
    let button = played.object("Button");
    let results = Rc::new(RefCell::new(Vec::new()));
    let log = results.clone();
    played.drive(move |_, scene| {
        let mut log = log.borrow_mut();
        log.push(scene.set_number(&SceneObject::root(100_000), 13, 1.0));
        log.push(scene.set_text(&button, TextValueRunBase::TEXT_PROPERTY_KEY, "words"));
        log.push(scene.set_number(&button, DISPLAY, 1.0));
        log.push(scene.set_number(
            &button,
            crate::source::generated::node_base::NodeBase::X_PROPERTY_KEY,
            f32::NAN,
        ));
    });
    let output = played.step(FRAME, vec![]);
    assert_eq!(
        *results.borrow(),
        [
            Err(SceneWriteError::NotFound),
            Err(SceneWriteError::UnknownProperty),
            Err(SceneWriteError::UnknownProperty),
            Err(SceneWriteError::InvalidValue),
        ]
    );
    assert_eq!(output.driver_rounds, 1);
}

/// A driver that writes nothing leaves every step as a player without one
/// steps it: the same reports, scheduling and draw.
#[test]
fn a_quiet_driver_steps_as_no_driver() {
    let run = |driven: bool| {
        let mut played = Played::new(hide_column());
        if driven {
            played.drive(|_, _| {});
        }
        let outputs = [
            played.step(FRAME, vec![]),
            played.tap(50.0, 20.0),
            played.tap(50.0, 60.0),
            played.step(FRAME, vec![]),
        ]
        .map(|output| {
            (
                names(&output.events),
                output.keep_going,
                output.dirty,
                output.settled,
            )
        });
        (outputs, drawn_paths(&played.f))
    };
    assert_eq!(run(true), run(false));
}

/// Reset puts typed text back to the text last written to the field, here a
/// host's, and leaves that written text alone.
#[test]
fn reset_returns_typed_text_to_the_written_text() {
    use crate::source::generated::text::text_input_base::TextInputBase;
    let mut played = Played::new(scene(
        (400.0, 300.0),
        css(
            "Page",
            [width(400.0), height(300.0)].concat(),
            vec![
                field_box(true, None, "hello world"),
                B {
                    listener: Some("clear"),
                    opaque: true,
                    ..css("Clear", [width(100.0), height(40.0)].concat(), vec![])
                },
            ],
        ),
    ));
    played.settle();
    let field = played.f.object("Field");
    let field_id = SceneObject::root(played.local_id(&field));
    let page = played.object("Page");
    let text = move || {
        CoreRegistry::get_string_handle(&field, i32::from(TextInputBase::TEXT_PROPERTY_KEY))
            .expect("the field's text")
    };
    let steps = Rc::new(RefCell::new(0usize));
    let count = steps.clone();
    played.drive(move |round, scene| {
        if round.index != 0 {
            return;
        }
        *count.borrow_mut() += 1;
        if *count.borrow() == 1 {
            assert_eq!(
                scene.set_text(&field_id, TextInputBase::TEXT_PROPERTY_KEY, "hi there"),
                Ok(true)
            );
        }
        if names(round.reports).iter().any(|name| name == "clear") {
            assert_eq!(scene.reset(&page), Ok(true));
        }
    });
    played.step(FRAME, vec![]);
    assert_eq!(text(), "hi there");
    // What a focused field does with typed text.
    assert_eq!(
        played
            .f
            .object("Field")
            .with_downcast_mut::<crate::source::text::text_input::TextInput, _>(|input| {
                input.text_input("!")
            }),
        Some(true)
    );
    assert_ne!(text(), "hi there", "typed into the field");
    played.tap(50.0, 60.0);
    assert_eq!(text(), "hi there");
}
