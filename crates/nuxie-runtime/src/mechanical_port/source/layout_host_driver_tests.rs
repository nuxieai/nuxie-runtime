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
    self, DriverFault, DriverRound, ErrorKind, HostDriver, MAX_DRIVER_ROUNDS, OwnedPlayerEvent,
    PagePlayer, Playback, Pointer, PointerKind, SceneObject, SceneWriteError, SceneWriter, Step,
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
        Self::start(f, false)
    }

    /// As `new`, with a new instance of the file's view model bound to the
    /// artboard and its machine before the first frame.
    fn bound(f: Scene) -> Self {
        Self::start(f, true)
    }

    fn start(f: Scene, bound: bool) -> Self {
        let mut board =
            ArtboardInstance::from_native_handle(f._file.clone(), 0, f.artboard.clone());
        let machine = board.state_machine_instance(0).expect("the state machine");
        if bound {
            let instance = f
                ._file
                .with_file(|file| {
                    file.create_view_model_instance_for_artboard(f.artboard.core_handle())
                })
                .expect("a view model instance");
            f.artboard.bind_view_model_instance(Some(instance.clone()));
            machine
                .native_handle()
                .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        }
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
        self.try_step(elapsed_seconds, pointers).expect("the step")
    }

    fn try_step(
        &mut self,
        elapsed_seconds: f32,
        pointers: Vec<Pointer>,
    ) -> Result<Output, page_player::Error> {
        self.player.step(
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

    /// The frames it takes the player to say it is settled.
    fn frames_until_settled(&mut self) -> usize {
        for frame in 1..=240 {
            if self.step(FRAME, vec![]).settled {
                return frame;
            }
        }
        panic!("the player never settled");
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

/// What one step's result may hold in owned bytes (`page_player`'s bound).
const STEP_RESULT_BYTES: usize = 8 * 1024 * 1024;

/// A root artboard hosting "Nested", whose machine fires an event named
/// `name` as it enters its one state, which plays a half-second animation
/// moving the nested "Dot". Resetting "Nested" returns that machine to its
/// entry, so the settle after a reset fires the event again and the
/// animation starts over.
fn reporting_nested(name: &str) -> Scene {
    use crate::source::generated::{
        animation::{
            animation_state_base::AnimationStateBase, any_state_base::AnyStateBase,
            entry_state_base::EntryStateBase, exit_state_base::ExitStateBase,
            interpolating_keyframe_base::InterpolatingKeyFrameBase,
            keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
            keyframe_base::KeyFrameBase, keyframe_double_base::KeyFrameDoubleBase,
            linear_animation_base::LinearAnimationBase,
            nested_state_machine_base::NestedStateMachineBase,
            state_machine_base::StateMachineBase,
            state_machine_fire_event_base::StateMachineFireEventBase,
            state_machine_layer_base::StateMachineLayerBase,
            state_transition_base::StateTransitionBase,
        },
        event_base::EventBase,
        nested_animation_base::NestedAnimationBase,
        nested_artboard_base::NestedArtboardBase,
    };
    // Layer states: 0 entry, 1 any, 2 exit, then the one that fires.
    let machine = |riv: &mut Riv, fires: Option<u64>| {
        riv.object(StateMachineBase::TYPE_KEY).end();
        riv.object(StateMachineLayerBase::TYPE_KEY).end();
        riv.object(EntryStateBase::TYPE_KEY).end();
        if fires.is_some() {
            riv.object(StateTransitionBase::TYPE_KEY)
                .uint(StateTransitionBase::STATE_TO_ID_PROPERTY_KEY, 3)
                .end();
        }
        riv.object(AnyStateBase::TYPE_KEY).end();
        riv.object(ExitStateBase::TYPE_KEY).end();
        if let Some(event) = fires {
            riv.object(AnimationStateBase::TYPE_KEY)
                .uint(AnimationStateBase::ANIMATION_ID_PROPERTY_KEY, 0)
                .end();
            riv.object(StateMachineFireEventBase::TYPE_KEY)
                .uint(StateMachineFireEventBase::EVENT_ID_PROPERTY_KEY, event)
                .end();
        }
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // 0 artboard, 1 host, 2 its machine.
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Main")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .end();
    riv.object(NestedArtboardBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Nested")
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
        .end();
    riv.object(NestedStateMachineBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
        .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 0)
        .end();
    machine(&mut riv, None);
    // 0 artboard, 1 the event, 2 the dot.
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Child")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .end();
    riv.object(EventBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, name)
        .end();
    riv.object(crate::source::generated::node_base::NodeBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Dot")
        .end();
    riv.object(LinearAnimationBase::TYPE_KEY)
        .uint(LinearAnimationBase::FPS_PROPERTY_KEY, 60)
        .uint(LinearAnimationBase::DURATION_PROPERTY_KEY, 30)
        .end();
    riv.object(KeyedObjectBase::TYPE_KEY)
        .uint(KeyedObjectBase::OBJECT_ID_PROPERTY_KEY, 2)
        .end();
    riv.object(KeyedPropertyBase::TYPE_KEY)
        .uint(
            KeyedPropertyBase::PROPERTY_KEY_PROPERTY_KEY,
            u64::from(crate::source::generated::node_base::NodeBase::X_PROPERTY_KEY),
        )
        .end();
    for (frame, value) in [(0, 0.0), (30, 100.0)] {
        riv.object(KeyFrameDoubleBase::TYPE_KEY)
            .uint(KeyFrameBase::FRAME_PROPERTY_KEY, frame)
            .float(KeyFrameDoubleBase::VALUE_PROPERTY_KEY, value)
            .uint(
                InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY,
                1,
            )
            .end();
    }
    machine(&mut riv, Some(1));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the reporting scene imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("the outer artboard");
    Scene {
        _file: file,
        artboard,
        factory,
    }
}

/// A driver that resets "Nested" in every round, so every settle reports
/// the nested event again. The name bytes of each report it was handed.
fn resetting(played: &Played) -> Rc<RefCell<Vec<usize>>> {
    let nested = played.object("Nested");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = seen.clone();
    played.drive(move |round, scene| {
        log.borrow_mut().extend(
            round
                .reports
                .iter()
                .map(|report| report.name.as_deref().map_or(0, <[u8]>::len)),
        );
        assert_eq!(scene.reset(&nested), Ok(true));
    });
    seen
}

/// One budget covers the whole step. Driver rounds whose reports are each
/// well under the step's byte bound, and together over it, fail the step
/// with a limit error, and the driver is never handed the report that
/// crosses the bound.
#[test]
fn rounds_each_under_the_byte_bound_and_together_over_it_fail_the_step() {
    let size = STEP_RESULT_BYTES / 6 + 1;
    let mut played = Played::new(reporting_nested(&"r".repeat(size)));
    let first = played.step(FRAME, vec![]);
    assert_eq!(
        names(&first.events).iter().map(String::len).sum::<usize>(),
        0,
        "the nested machine entered its state when the scene began"
    );
    let seen = resetting(&played);
    let error = played
        .try_step(FRAME, vec![])
        .err()
        .expect("six reports of a sixth of the bound each exceed it");
    assert_eq!(error.kind, ErrorKind::LimitExceeded);
    let seen = seen.borrow();
    assert_eq!(
        *seen,
        vec![size; 5],
        "the sixth report never reaches the driver"
    );
    assert!(seen.iter().sum::<usize>() <= STEP_RESULT_BYTES);
}

/// The reports a driver cut off at the bound carries into the next step
/// are charged to that step's budget before its first round, beside the
/// reports that step's own rounds make.
#[test]
fn reports_carried_past_a_cut_off_count_toward_the_next_step() {
    // Eight fit in one step, nine do not.
    let size = STEP_RESULT_BYTES * 2 / 17;
    let mut played = Played::new(reporting_nested(&"c".repeat(size)));
    played.step(FRAME, vec![]);
    let seen = resetting(&played);
    let cut = played.step(FRAME, vec![]);
    assert_eq!(cut.driver_rounds, MAX_DRIVER_ROUNDS);
    assert!(!cut.settled, "the driver was cut off at the bound");
    assert_eq!(cut.events.len(), MAX_DRIVER_ROUNDS);
    assert_eq!(seen.borrow().len(), MAX_DRIVER_ROUNDS - 1);
    seen.borrow_mut().clear();
    let error = played
        .try_step(FRAME, vec![])
        .err()
        .expect("the carried report and eight more exceed the bound");
    assert_eq!(error.kind, ErrorKind::LimitExceeded);
    let seen = seen.borrow();
    assert_eq!(
        *seen,
        vec![size; MAX_DRIVER_ROUNDS],
        "the carried report, then seven rounds' reports; the ninth never reaches the driver"
    );
}

/// A still player whose driver resets a nested machine, restarting its
/// half-second entry animation, says the step is not settled; the frames
/// after it advance the animation to its end and then settle.
#[test]
fn a_reset_that_restarts_a_timed_animation_leaves_the_step_unsettled() {
    let mut played = Played::new(reporting_nested("entered"));
    assert!(
        played.frames_until_settled() > 20,
        "the entry animation plays out before the player is still"
    );
    let nested = played.object("Nested");
    let resets = Rc::new(RefCell::new(0));
    let count = resets.clone();
    played.drive(move |_, scene| {
        if *count.borrow() == 0 {
            *count.borrow_mut() += 1;
            assert_eq!(scene.reset(&nested), Ok(true));
        }
    });
    let reset = played.step(FRAME, vec![]);
    assert_eq!(
        names(&reset.events),
        ["entered"],
        "the machine entered again"
    );
    assert!(!reset.settled, "the restarted animation is pending");
    assert!(reset.dirty);
    let frames = played.frames_until_settled();
    assert!(
        frames > 20,
        "the frames after the reset play the animation out: {frames}"
    );
    let quiet = played.step(FRAME, vec![]);
    assert!(quiet.settled && !quiet.dirty);
}

/// A view model's number, two-way bound to the root's "Mover" x, and a root
/// machine that moves from its animation state to a blend state when that
/// number is 100.
fn bound_transition() -> Scene {
    use crate::source::data_bind::data_bind::TWO_WAY;
    use crate::source::generated::{
        animation::{
            animation_state_base::AnimationStateBase, any_state_base::AnyStateBase,
            blend_state_direct_base::BlendStateDirectBase, entry_state_base::EntryStateBase,
            exit_state_base::ExitStateBase, linear_animation_base::LinearAnimationBase,
            state_machine_base::StateMachineBase, state_machine_layer_base::StateMachineLayerBase,
            state_transition_base::StateTransitionBase,
            transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
            transition_value_number_comparator_base::TransitionValueNumberComparatorBase,
            transition_viewmodel_condition_base::TransitionViewModelConditionBase,
        },
        data_bind::{
            bindable_property_number_base::BindablePropertyNumberBase,
            data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase,
        },
        viewmodel::{
            viewmodel_base::ViewModelBase, viewmodel_component_base::ViewModelComponentBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        },
    };
    let bind = |riv: &mut Riv, key: u16, flags: u32| {
        riv.object(DataBindContextBase::TYPE_KEY)
            .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, u64::from(key))
            .uint(DataBindBase::FLAGS_PROPERTY_KEY, u64::from(flags))
            .byte_field(DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY, &[0, 0])
            .end();
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ViewModelBase::TYPE_KEY)
        .string(ViewModelComponentBase::NAME_PROPERTY_KEY, "Model")
        .end();
    riv.object(ViewModelPropertyNumberBase::TYPE_KEY)
        .string(ViewModelComponentBase::NAME_PROPERTY_KEY, "n")
        .end();
    // 0 artboard, 1 the mover.
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Main")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(NodeBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Mover")
        .end();
    bind(&mut riv, NodeBase::X_PROPERTY_KEY, TWO_WAY);
    riv.object(LinearAnimationBase::TYPE_KEY)
        .uint(LinearAnimationBase::FPS_PROPERTY_KEY, 60)
        .uint(LinearAnimationBase::DURATION_PROPERTY_KEY, 60)
        .end();
    // Layer states: 0 entry, 1 any, 2 exit, 3 the animation, 4 the blend.
    riv.object(StateMachineBase::TYPE_KEY).end();
    riv.object(StateMachineLayerBase::TYPE_KEY).end();
    riv.object(EntryStateBase::TYPE_KEY).end();
    riv.object(StateTransitionBase::TYPE_KEY)
        .uint(StateTransitionBase::STATE_TO_ID_PROPERTY_KEY, 3)
        .end();
    riv.object(AnyStateBase::TYPE_KEY).end();
    riv.object(ExitStateBase::TYPE_KEY).end();
    riv.object(AnimationStateBase::TYPE_KEY)
        .uint(AnimationStateBase::ANIMATION_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(StateTransitionBase::TYPE_KEY)
        .uint(StateTransitionBase::STATE_TO_ID_PROPERTY_KEY, 4)
        .end();
    riv.object(BindablePropertyNumberBase::TYPE_KEY).end();
    bind(
        &mut riv,
        BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
        0,
    );
    riv.object(TransitionViewModelConditionBase::TYPE_KEY)
        .uint(TransitionViewModelConditionBase::OP_VALUE_PROPERTY_KEY, 0)
        .end();
    riv.object(TransitionPropertyViewModelComparatorBase::TYPE_KEY)
        .end();
    riv.object(TransitionValueNumberComparatorBase::TYPE_KEY)
        .float(
            TransitionValueNumberComparatorBase::VALUE_PROPERTY_KEY,
            100.0,
        )
        .end();
    riv.object(BlendStateDirectBase::TYPE_KEY).end();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the bound scene imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("the artboard");
    Scene {
        _file: file,
        artboard,
        factory,
    }
}

/// The core types of a step's changed states.
fn state_types(output: &Output) -> Vec<u32> {
    output
        .states
        .iter()
        .map(|state| state.state_core_type)
        .collect()
}

/// A driver's write through a two-way bind moves the bound number a root
/// transition reads; the settle takes the transition, and that state change
/// is in the same step's output, as it is when the same write is made
/// between frames and the next frame takes it.
#[test]
fn a_transition_a_driver_write_causes_is_in_the_step_states() {
    use crate::source::generated::animation::blend_state_direct_base::BlendStateDirectBase;
    let blend = vec![u32::from(BlendStateDirectBase::TYPE_KEY)];
    // The oracle: the write made between frames.
    let mut oracle = Played::bound(bound_transition());
    oracle.step(FRAME, vec![]);
    assert!(CoreRegistry::set_double_handle(
        &oracle.f.object("Mover"),
        i32::from(NodeBase::X_PROPERTY_KEY),
        100.0
    ));
    assert_eq!(state_types(&oracle.step(FRAME, vec![])), blend);
    assert!(oracle.step(FRAME, vec![]).states.is_empty());

    let mut played = Played::bound(bound_transition());
    played.step(FRAME, vec![]);
    let mover = played.object("Mover");
    let writes = Rc::new(RefCell::new(0));
    let count = writes.clone();
    played.drive(move |_, scene| {
        if *count.borrow() == 0 {
            *count.borrow_mut() += 1;
            assert_eq!(
                scene.set_number(&mover, NodeBase::X_PROPERTY_KEY, 100.0),
                Ok(true)
            );
        }
    });
    let written = played.step(FRAME, vec![]);
    assert_eq!(
        state_types(&written),
        blend,
        "the transition the write caused"
    );
    assert!(!written.settled);
    assert!(played.step(FRAME, vec![]).states.is_empty(), "taken once");
}

/// A column: a box that resizes over a second when its layout changes, and
/// a box below it.
fn interpolating_column() -> Scene {
    use crate::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase as Style;
    const CUSTOM: u64 = 2;
    const LINEAR: u64 = 1;
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [width(400.0), height(300.0)].concat(),
            vec![
                B {
                    style: vec![
                        U(Style::ANIMATION_STYLE_TYPE_PROPERTY_KEY, CUSTOM),
                        U(Style::INTERPOLATION_TYPE_PROPERTY_KEY, LINEAR),
                        F(Style::INTERPOLATION_TIME_PROPERTY_KEY, 1.0),
                    ],
                    listener: Some("tap"),
                    ..css("Card", [width(100.0), height(40.0)].concat(), vec![])
                },
                css("Below", [width(100.0), height(40.0)].concat(), vec![]),
            ],
        ),
    )
}

/// A driver's write that begins a timed layout transition leaves its step
/// unsettled; the frames after it resize the box over the transition's
/// second and then settle at the written width.
#[test]
fn a_write_that_begins_a_layout_transition_leaves_the_step_unsettled() {
    let mut played = Played::new(interpolating_column());
    played.frames_until_settled();
    let record = played.f.record("Card");
    let card = SceneObject::root(played.local_id(&record));
    let writes = Rc::new(RefCell::new(0));
    let count = writes.clone();
    played.drive(move |_, scene| {
        if *count.borrow() == 0 {
            *count.borrow_mut() += 1;
            assert_eq!(scene.set_number(&card, WIDTH, 200.0), Ok(true));
        }
    });
    let written = played.step(FRAME, vec![]);
    assert!(
        !written.settled,
        "the transition the write began is pending"
    );
    let widths = (0..30)
        .map(|_| {
            let output = played.step(FRAME, vec![]);
            (played.f.bounds("Card")[2], output.settled)
        })
        .collect::<Vec<_>>();
    let (midway, settled) = widths[29];
    assert!(
        midway > 110.0 && midway < 190.0 && !settled,
        "half a second into the resize: {widths:?}"
    );
    played.frames_until_settled();
    assert!(near(played.f.bounds("Card")[2], 200.0));
}

/// What hosts the "Child" artboard in `hosting`.
#[derive(Clone, Copy, Debug)]
enum Host {
    /// A nested artboard of this host type, playing the child's machine
    /// and, as a simple animation, its second animation.
    Nested(u16),
    /// The one row of an artboard component list, with the row's machine.
    ListRow,
}

/// Every kind of host: each nested artboard host type, and a list row.
const HOSTS: [Host; 4] = [
    Host::Nested(crate::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY),
    Host::Nested(
        crate::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY,
    ),
    Host::Nested(
        crate::source::generated::nested_artboard_leaf_base::NestedArtboardLeafBase::TYPE_KEY,
    ),
    Host::ListRow,
];

/// A root "Main" whose "Box" holds `host`, hosting "Child": a machine that
/// fires "entered" as it enters a half-second animation moving the drawn
/// "Dot", a second half-second animation moving the drawn "Bar", a
/// horizontal scroll view whose 400 point list, holding a drawn tile, sits
/// in a 200 point viewport, and a drawn field the file gives the text
/// "hello world".
fn hosting(host: Host) -> Scene {
    use crate::source::{
        artboard::Artboard,
        artboard_component_list::ArtboardComponentList,
        generated::{
            animation::{
                animation_state_base::AnimationStateBase, any_state_base::AnyStateBase,
                entry_state_base::EntryStateBase, exit_state_base::ExitStateBase,
                interpolating_keyframe_base::InterpolatingKeyFrameBase,
                keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
                keyframe_base::KeyFrameBase, keyframe_double_base::KeyFrameDoubleBase,
                linear_animation_base::LinearAnimationBase,
                nested_simple_animation_base::NestedSimpleAnimationBase,
                nested_state_machine_base::NestedStateMachineBase,
                state_machine_base::StateMachineBase,
                state_machine_fire_event_base::StateMachineFireEventBase,
                state_machine_layer_base::StateMachineLayerBase,
                state_transition_base::StateTransitionBase,
            },
            artboard_component_list_base::ArtboardComponentListBase,
            assets::{
                file_asset_base::FileAssetBase, file_asset_contents_base::FileAssetContentsBase,
                font_asset_base::FontAssetBase,
            },
            constraints::{
                draggable_constraint_base::DraggableConstraintBase,
                scrolling::scroll_constraint_base::ScrollConstraintBase,
            },
            event_base::EventBase,
            focus_data_base::FocusDataBase,
            nested_animation_base::NestedAnimationBase,
            nested_artboard_base::NestedArtboardBase,
            nested_artboard_leaf_base::NestedArtboardLeafBase,
            text::{
                text_input_base::TextInputBase, text_style_base::TextStyleBase,
                text_style_paint_base::TextStylePaintBase,
            },
        },
        viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
    };
    // Layer states: 0 entry, 1 any, 2 exit, then the one that fires.
    let machine = |riv: &mut Riv, fires: Option<u64>| {
        riv.object(StateMachineBase::TYPE_KEY).end();
        riv.object(StateMachineLayerBase::TYPE_KEY).end();
        riv.object(EntryStateBase::TYPE_KEY).end();
        if fires.is_some() {
            riv.object(StateTransitionBase::TYPE_KEY)
                .uint(StateTransitionBase::STATE_TO_ID_PROPERTY_KEY, 3)
                .end();
        }
        riv.object(AnyStateBase::TYPE_KEY).end();
        riv.object(ExitStateBase::TYPE_KEY).end();
        if let Some(event) = fires {
            riv.object(AnimationStateBase::TYPE_KEY)
                .uint(AnimationStateBase::ANIMATION_ID_PROPERTY_KEY, 0)
                .end();
            riv.object(StateMachineFireEventBase::TYPE_KEY)
                .uint(StateMachineFireEventBase::EVENT_ID_PROPERTY_KEY, event)
                .end();
        }
    };
    let style = |riv: &mut Riv, parent: u64, column: bool| {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        write(riv, &point_units());
        if column {
            write(
                riv,
                &[U(
                    LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                    YGFlexDirection::Column as u64,
                )],
            );
        }
        riv.end();
    };
    let layout = |riv: &mut Riv, parent: u64, name: &str, size: (f32, f32), style: u64| {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .string(ComponentBase::NAME_PROPERTY_KEY, name)
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, size.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, size.1)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, style)
            .end();
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(FontAssetBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "font")
        .uint(FileAssetBase::ASSET_ID_PROPERTY_KEY, 2)
        .end();
    riv.object(FileAssetContentsBase::TYPE_KEY)
        .byte_field(
            FileAssetContentsBase::BYTES_PROPERTY_KEY,
            include_bytes!("../../../../../fixtures/fonts/roboto-input.ttf"),
        )
        .end();
    // Main: 0 artboard, 1 the box, 2 the host, then a nested host's 3
    // machine and 4 simple animation, then the two styles.
    let first_style = match host {
        Host::Nested(_) => 5,
        Host::ListRow => 3,
    };
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Main")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 400.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style)
        .end();
    layout(&mut riv, 0, "Box", (200.0, 100.0), first_style + 1);
    match host {
        Host::Nested(type_key) => {
            riv.object(type_key)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
                .string(ComponentBase::NAME_PROPERTY_KEY, "Nested")
                .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1);
            if type_key == NestedArtboardLeafBase::TYPE_KEY {
                riv.uint(NestedArtboardLeafBase::FIT_PROPERTY_KEY, 1);
            }
            riv.end();
            riv.object(NestedStateMachineBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
                .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 0)
                .end();
            riv.object(NestedSimpleAnimationBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
                .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 1)
                .float(NestedSimpleAnimationBase::SPEED_PROPERTY_KEY, 1.0)
                .bool(NestedSimpleAnimationBase::IS_PLAYING_PROPERTY_KEY, true)
                .end();
        }
        Host::ListRow => {
            riv.object(ArtboardComponentListBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
                .string(ComponentBase::NAME_PROPERTY_KEY, "Rows")
                .end();
        }
    }
    style(&mut riv, 0, false);
    style(&mut riv, 1, false);
    machine(&mut riv, None);
    // A drawn 10 point square of `color` named `name`: the shape, at local
    // id `shape`, its rectangle, its fill and the fill's color.
    let square = |riv: &mut Riv, parent: u64, shape: u64, name: &str, color: u32| {
        riv.object(ShapeBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .string(ComponentBase::NAME_PROPERTY_KEY, name)
            .end();
        riv.object(RectangleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, shape)
            .float(ParametricPathBase::WIDTH_PROPERTY_KEY, 10.0)
            .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0)
            .end();
        riv.object(FillBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, shape)
            .end();
        riv.object(SolidColorBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, shape + 2)
            .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, color)
            .end();
    };
    // Child: 0 artboard, 1 the event, 2 to 5 the dot, 6 to 9 the bar,
    // 10 the viewport, 11 the list, 12 its scroll constraint, 13 a filled
    // tile box in the list and 14 and 15 its fill, 16 the field's box, 17
    // the field, 18 its focus data, 19 its text style, 20 the drawable that
    // draws its text and 21 and 22 that drawable's fill, 23 to 27 the
    // styles.
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Child")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 23)
        .end();
    riv.object(EventBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "entered")
        .end();
    square(&mut riv, 0, 2, "Dot", 0xff11_2233);
    square(&mut riv, 0, 6, "Bar", 0xff44_5566);
    layout(&mut riv, 0, "Viewport", (200.0, 40.0), 24);
    layout(&mut riv, 10, "List", (400.0, 40.0), 25);
    riv.object(ScrollConstraintBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 11)
        .uint(DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY, 0)
        .end();
    // The scroll moves the list's layout children, so the tile is a box.
    layout(&mut riv, 11, "Tile", (40.0, 40.0), 26);
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 13)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 14)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xff77_8899)
        .end();
    layout(&mut riv, 0, "Field box", (180.0, 40.0), 27);
    riv.object(TextInputBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 16)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Field")
        .string(TextInputBase::TEXT_PROPERTY_KEY, "hello world")
        .end();
    riv.object(FocusDataBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 17)
        .end();
    riv.object(TextStylePaintBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 17)
        .uint(TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, 0)
        .float(TextStyleBase::FONT_SIZE_PROPERTY_KEY, 16.0)
        .end();
    riv.object(crate::source::generated::text::text_input_text_base::TextInputTextBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 17)
        .end();
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 20)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 21)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xff00_0000)
        .end();
    style(&mut riv, 0, true);
    style(&mut riv, 10, true);
    style(&mut riv, 11, false);
    style(&mut riv, 13, false);
    style(&mut riv, 16, false);
    // 0 moves the dot and 1 the bar, from 0 to 100 over half a second.
    for keyed in [2, 6] {
        riv.object(LinearAnimationBase::TYPE_KEY)
            .uint(LinearAnimationBase::FPS_PROPERTY_KEY, 60)
            .uint(LinearAnimationBase::DURATION_PROPERTY_KEY, 30)
            .end();
        riv.object(KeyedObjectBase::TYPE_KEY)
            .uint(KeyedObjectBase::OBJECT_ID_PROPERTY_KEY, keyed)
            .end();
        riv.object(KeyedPropertyBase::TYPE_KEY)
            .uint(
                KeyedPropertyBase::PROPERTY_KEY_PROPERTY_KEY,
                u64::from(NodeBase::X_PROPERTY_KEY),
            )
            .end();
        for (frame, value) in [(0, 0.0), (30, 100.0)] {
            riv.object(KeyFrameDoubleBase::TYPE_KEY)
                .uint(KeyFrameBase::FRAME_PROPERTY_KEY, frame)
                .float(KeyFrameDoubleBase::VALUE_PROPERTY_KEY, value)
                .uint(
                    InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY,
                    1,
                )
                .end();
        }
    }
    machine(&mut riv, Some(1));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the hosting scene imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("the outer artboard");
    if let Host::ListRow = host {
        // Mount one row, as a bound list of one item does.
        let list = artboard
            .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone());
        let item = list
            .retain_arena()
            .expect("list arena")
            .insert(ViewModelInstanceListItem::default());
        ArtboardComponentList::update_list_occurrence(&list, &[item]);
        let source = file
            .with_file(|file| file.artboard_at_source(1))
            .expect("row source");
        let row = Artboard::nested_instance_from_handle(&source).expect("row instance");
        ArtboardComponentList::add_artboard_at_occurrence(&list, row, 0, true);
    }
    Scene {
        _file: file,
        artboard,
        factory,
    }
}

/// The artboard `host` hosts in a `hosting` scene.
fn hosted_board(played: &Played, host: Host) -> RuntimeArtboardInstanceHandle {
    use crate::source::artboard_component_list::ArtboardComponentList;
    match host {
        Host::Nested(_) => played
            .f
            .object("Nested")
            .with(|host| {
                host.as_nested_artboard()
                    .and_then(|nested| nested.artboard_instance_default())
            })
            .flatten(),
        Host::ListRow => played
            .f
            .object("Rows")
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(0))
            .flatten(),
    }
    .expect("the hosted artboard")
}

/// The hosted machine of a `hosting` scene: a nested host's nested state
/// machine, or the list row's machine.
fn hosted_machine(played: &Played, host: Host) -> crate::RuntimeStateMachineInstanceHandle {
    use crate::source::{
        animation::nested_state_machine::NestedStateMachine,
        artboard_component_list::ArtboardComponentList,
    };
    match host {
        Host::Nested(_) => played
            .f
            .object("Nested")
            .with(|host| {
                host.as_nested_artboard()
                    .map(|nested| nested.nested_animations()[0].clone())
            })
            .flatten()
            .and_then(|machine| {
                machine
                    .with_downcast::<NestedStateMachine, _>(
                        NestedStateMachine::state_machine_instance,
                    )
                    .flatten()
            }),
        Host::ListRow => played
            .f
            .object("Rows")
            .with_downcast::<ArtboardComponentList, _>(|list| list.state_machine_instance(0))
            .flatten(),
    }
    .expect("the hosted machine")
}

/// What a `hosting` scene's player holds and shows: the hosted artboard's
/// animated x of "Dot" and "Bar", its scroll offset, the list's world
/// bounds and where the tile the list scrolls lands, the field's text, its machine's state and animation, the host's
/// written width, and the whole draw of the artboard, transforms and paths.
#[derive(Debug, PartialEq)]
struct Held {
    dot_x: f32,
    bar_x: f32,
    scroll: (f32, f32),
    list_bounds: [f32; 4],
    tile_at: (f32, f32),
    text: String,
    state: Option<String>,
    animation: Option<(String, f32)>,
    box_width: Option<f32>,
    drawn: Vec<String>,
}

fn held(played: &Played, host: Host) -> Held {
    use crate::source::generated::text::text_input_base::TextInputBase;
    let board = hosted_board(played, host);
    let object = |local_id: usize| {
        board
            .with_artboard(|artboard| artboard.objects()[local_id].clone())
            .expect("a hosted object")
    };
    let x = |local_id| {
        CoreRegistry::get_double_handle(&object(local_id), i32::from(NodeBase::X_PROPERTY_KEY))
            .expect("an x")
    };
    let list_bounds = object(11)
        .with_downcast::<LayoutComponent, _>(|list| {
            let bounds = list.world_bounds();
            [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
        })
        .expect("the list");
    let machine = hosted_machine(played, host);
    let state = machine
        .with_instance_mut(|machine| machine.layer_state(0))
        .map(|state| format!("{:?}", state.core_type()));
    let animation = machine
        .with_instance_mut(|machine| machine.current_animation_by_index(0))
        .and_then(|state| {
            state.first_animation(|animation| (animation.name().to_string(), animation.time()))
        });
    Held {
        dot_x: x(2),
        bar_x: x(6),
        scroll: object(12)
            .with_downcast(|scroll: &ScrollConstraint| (scroll.offset_x(), scroll.offset_y()))
            .expect("a live scroll constraint"),
        list_bounds,
        tile_at: object(13)
            .with(|tile| {
                tile.as_world_transform_component().map(|tile| {
                    let world = tile.world_transform();
                    (world.tx(), world.ty())
                })
            })
            .flatten()
            .expect("the tile"),
        text: CoreRegistry::get_string_handle(
            &object(17),
            i32::from(TextInputBase::TEXT_PROPERTY_KEY),
        )
        .expect("the field's text"),
        state,
        animation,
        box_width: CoreRegistry::get_double_handle(
            &played.f.object("Box"),
            i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
        ),
        // What the renderer is asked to draw: the factory's own lines, which
        // make the paths it draws the first time, are left out.
        drawn: played
            .f
            .draw_lines()
            .iter()
            .filter(|line| !line.starts_with("make"))
            .map(|line| without_ids(line))
            .collect(),
    }
}

/// A draw line without the factory's handle numbers, which count every
/// path and paint the factory has made, so two copies made at different
/// times number the same draw differently.
fn without_ids(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("id=") {
        out.push_str(&rest[..at + 3]);
        rest = rest[at + 3..].trim_start_matches(|c: char| c.is_ascii_digit());
    }
    out.push_str(rest);
    out
}

/// The width a host writes to "Box" before it moves anything.
const HOST_WIDTH: f32 = 180.0;

/// The reset contract, for one host kind and one reset object: after a
/// reset inside a step, everything the engine holds under that object is
/// what a freshly made instance of the same file holds after its first
/// settle, and the frame drawn after that same step shows it. Property
/// values a host wrote are the only difference, and they stay.
///
/// The played copy takes a host write, then moves everything it holds: its
/// machine plays its entry animation to the end, a nested host's simple
/// animation plays to its end, the scroll view scrolls and the field takes
/// typed text. A driver then resets `target` (the root when None) inside a
/// step. That same step's result is compared, with no frame between, with a
/// second copy of the file made fresh, given the same host write, and
/// stepped once.
fn reset_is_a_fresh_instance(host: Host, target: Option<&str>) {
    use crate::source::{
        generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        text::text_input::TextInput,
    };
    let fresh = {
        let f = hosting(host);
        assert!(CoreRegistry::set_double_handle(
            &f.object("Box"),
            i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
            HOST_WIDTH
        ));
        Played::new(f)
    };
    let expected = held(&fresh, host);

    let mut played = Played::new(hosting(host));
    let target = match target {
        Some(name) => played.object(name),
        None => SceneObject::root(played.local_id(&played.f.artboard.core_handle())),
    };
    let box_id = played.object("Box");
    let resetting = Rc::new(RefCell::new(false));
    let result = Rc::new(RefCell::new(None));
    let (armed, log) = (resetting.clone(), result.clone());
    played.drive(move |round, scene| {
        if round.index != 0 {
            return;
        }
        if *armed.borrow() {
            *armed.borrow_mut() = false;
            *log.borrow_mut() = Some(scene.reset(&target));
        } else {
            let _ = scene.set_number(&box_id, LayoutComponentBase::WIDTH_PROPERTY_KEY, HOST_WIDTH);
        }
    });
    played.frames_until_settled();
    let moved = held(&played, host);
    assert_eq!(
        moved.box_width,
        Some(HOST_WIDTH),
        "{host:?}: the host wrote"
    );
    assert_eq!(
        moved.dot_x, 100.0,
        "{host:?}: the entry animation played out"
    );
    if matches!(host, Host::Nested(_)) {
        assert_eq!(
            moved.bar_x, 100.0,
            "{host:?}: the simple animation played out"
        );
    }
    let board = hosted_board(&played, host);
    let hosted = |local_id: usize| {
        board
            .with_artboard(|artboard| artboard.objects()[local_id].clone())
            .expect("a hosted object")
    };
    // What a focused field does with typed text.
    assert_eq!(
        hosted(17).with_downcast_mut::<TextInput, _>(|input| input.text_input("!")),
        Some(true)
    );
    assert!(CoreRegistry::set_double_handle(
        &hosted(12),
        i32::from(ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY),
        -50.0
    ));
    played.settle();
    let moved = held(&played, host);
    assert_ne!(moved.text, expected.text, "{host:?}: typed into the field");
    assert_ne!(moved.scroll, expected.scroll, "{host:?}: scrolled");
    assert_ne!(moved.tile_at, expected.tile_at, "{host:?}: scrolled");
    assert_ne!(moved.drawn, expected.drawn, "{host:?}: the draw moved");

    *resetting.borrow_mut() = true;
    played.step(FRAME, vec![]);
    assert_eq!(*result.borrow(), Some(Ok(true)), "{host:?}");
    assert_eq!(held(&played, host), expected, "{host:?}");
}

#[test]
fn a_reset_of_a_box_holding_each_host_is_a_fresh_instance() {
    for host in HOSTS {
        reset_is_a_fresh_instance(host, Some("Box"));
    }
}

#[test]
fn a_reset_of_the_root_is_a_fresh_instance_for_each_host() {
    for host in HOSTS {
        reset_is_a_fresh_instance(host, None);
    }
}
