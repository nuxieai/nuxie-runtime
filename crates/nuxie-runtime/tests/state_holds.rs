//! State holds: a self-transition with Nuxie's `isHold` (60023) stands its
//! layer still while its conditions pass (docs/live-moves.md, State holds).
//!
//! A held layer is compared against a twin: a second instance of the same
//! file, given the same inputs except the hold's own, that makes a zero-second
//! advance wherever the held build makes a held advance. After release the two
//! agree exactly. While held, the oracle is the value read just before the
//! first held advance, not the twin, which applies. A plain run is never held.
#![cfg(feature = "testing")]

use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advancing_component::QuietState,
    animation::{
        state_machine::StateMachine,
        state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
        state_machine_layer::StateMachineLayer,
    },
    generated::core_registry::CoreRegistry,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_string::ViewModelInstanceString,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    live_move_fixture,
};
use serde::Deserialize;
use std::path::Path;

mod cpp_probe_support;
use cpp_probe_support::*;
#[path = "cpp_probe_support/transition_fixtures.rs"]
mod transition_fixtures;
use transition_fixtures::{
    push_blend_animation_1d, push_synthetic_bool_transition_condition,
    synthetic_state_machine_same_state_hold, synthetic_state_machine_same_state_hold_with_toc,
    synthetic_state_machine_same_state_transition,
};

// The probe observations test 11 compares. `cpp_probe_support` reads the
// probe's JSON into this type.
#[derive(Debug, Deserialize, PartialEq)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CppArtboard {
    #[serde(rename = "runtimeStateMachineAdvances", default)]
    runtime_state_machine_advances: Vec<CppRuntimeStateMachineAdvance>,
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CppRuntimeStateMachineAdvance {
    advanced: bool,
    #[serde(rename = "changedStateCount")]
    changed_state_count: usize,
    #[serde(rename = "currentAnimations")]
    current_animations: Vec<CppRuntimeStateMachineCurrentAnimation>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CppRuntimeStateMachineCurrentAnimation {
    time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CppRuntimeUpdate {
    components: Vec<CppRuntimeComponent>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
}

const DT: f32 = 1.0 / 60.0;

const DISABLED: u64 = 1 << 0;
const ENABLE_EXIT_TIME: u64 = 1 << 2;
const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
const PAUSE_ON_EXIT: u64 = 1 << 4;

// Layer state indices: Any and Entry come first, then the authored states.
const ANY: u64 = 0;
const FIRST: u64 = 2;

// Artboard-local ids.
const NODE: u32 = 1;
const PING: u64 = 2;
const ENTER: u64 = 3;
const START: u64 = 4;

#[derive(Clone, Copy)]
enum Anim {
    /// x from 0 to 100 over 1 s, one-shot.
    Move,
    /// x from 200 to 300 over 1 s, one-shot.
    Glide,
    /// x held at 50.
    Rest,
    /// x from 0 to 10 over 0.1 s, one-shot.
    Short,
}

impl Anim {
    fn id(self) -> u64 {
        match self {
            Anim::Move => 0,
            Anim::Glide => 1,
            Anim::Rest => 2,
            Anim::Short => 3,
        }
    }
}

#[derive(Clone, Copy)]
enum Input {
    Bool(&'static str),
    Trigger(&'static str),
    Number(&'static str),
}

#[derive(Clone, Copy)]
enum Cond {
    /// A state machine bool input is true.
    Bool(u64),
    /// A state machine trigger fired.
    Trigger(u64),
    /// The view model's step reference equals this value.
    Step(&'static str),
}

#[derive(Clone, Default)]
struct Tr {
    to: u64,
    hold: bool,
    blend: bool,
    flags: u64,
    duration: u64,
    exit_time: Option<u64>,
    conditions: Vec<Cond>,
    start_event: bool,
}

fn hold(state: u64, conditions: &[Cond]) -> Tr {
    Tr {
        to: state,
        hold: true,
        conditions: conditions.to_vec(),
        ..Tr::default()
    }
}

fn to(state: u64, conditions: &[Cond]) -> Tr {
    Tr {
        to: state,
        conditions: conditions.to_vec(),
        ..Tr::default()
    }
}

fn exit_at_end(state: u64) -> Tr {
    Tr {
        to: state,
        flags: ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
        exit_time: Some(100),
        ..Tr::default()
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Anim(Anim),
    /// A 1D blend of `move` (at 0) and `glide` (at 1) on a number input.
    Blend1D(u64),
}

struct St {
    kind: Kind,
    transitions: Vec<Tr>,
    enter_event: bool,
}

fn state(anim: Anim, transitions: Vec<Tr>) -> St {
    St {
        kind: Kind::Anim(anim),
        transitions,
        enter_event: false,
    }
}

#[derive(Default)]
struct Spec {
    view_model: bool,
    inputs: Vec<Input>,
    any: Vec<Tr>,
    states: Vec<St>,
    /// A KeyFrameCallback on `ping` in `move`, at this frame.
    move_event: Option<u64>,
}

fn is_hold_key() -> u16 {
    property_key_for_name("StateTransition", "isHold")
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_bytes_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &[u8]) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

/// The view-model compare of the walk's lifecycle guards
/// (src/live_move/device_fixture.rs `lifecycle_condition`), over these
/// writers: the step reference, view model 0 property 0, equals `value`.
fn push_step_condition(bytes: &mut Vec<u8>, value: &str) {
    push_object_with_properties(bytes, "BindablePropertyString", |_| {});
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyString",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &[0, 0]);
        push_uint_property(bytes, "DataBindContext", "flags", 0);
    });
    push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
        push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
    });
    push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
    push_object_with_properties(bytes, "TransitionValueStringComparator", |bytes| {
        push_string_property(bytes, "TransitionValueStringComparator", "value", value);
    });
}

fn push_condition(bytes: &mut Vec<u8>, condition: Cond) {
    match condition {
        Cond::Bool(input) => push_synthetic_bool_transition_condition(bytes, input),
        Cond::Trigger(input) => {
            push_object_with_properties(bytes, "TransitionTriggerCondition", |bytes| {
                push_uint_property(bytes, "TransitionTriggerCondition", "inputId", input);
            });
        }
        Cond::Step(value) => push_step_condition(bytes, value),
    }
}

/// A fire event at the start of the state or transition written before it.
fn push_fire_event(bytes: &mut Vec<u8>, event: u64) {
    push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
        push_uint_property(bytes, "StateMachineFireEvent", "eventId", event);
        push_uint_property(bytes, "StateMachineFireEvent", "occursValue", 0);
    });
}

fn push_transition(bytes: &mut Vec<u8>, transition: &Tr) {
    let type_name = if transition.blend {
        "BlendStateTransition"
    } else {
        "StateTransition"
    };
    push_object_with_properties(bytes, type_name, |bytes| {
        push_uint_property(bytes, type_name, "stateToId", transition.to);
        if transition.duration != 0 {
            push_uint_property(bytes, type_name, "duration", transition.duration);
        }
        if transition.flags != 0 {
            push_uint_property(bytes, type_name, "flags", transition.flags);
        }
        if let Some(exit_time) = transition.exit_time {
            push_uint_property(bytes, type_name, "exitTime", exit_time);
        }
        if transition.hold {
            push_bool_property(bytes, type_name, "isHold", true);
        }
    });
    for &condition in &transition.conditions {
        push_condition(bytes, condition);
    }
    if transition.start_event {
        push_fire_event(bytes, START);
    }
}

fn push_animation(
    bytes: &mut Vec<u8>,
    name: &str,
    duration: u64,
    keys: &[(u64, f32)],
    event_frame: Option<u64>,
) {
    push_object_with_properties(bytes, "LinearAnimation", |bytes| {
        push_string_property(bytes, "LinearAnimation", "name", name);
        push_uint_property(bytes, "LinearAnimation", "fps", 60);
        push_uint_property(bytes, "LinearAnimation", "duration", duration);
    });
    push_object_with_properties(bytes, "KeyedObject", |bytes| {
        push_uint_property(bytes, "KeyedObject", "objectId", u64::from(NODE));
    });
    push_object_with_properties(bytes, "KeyedProperty", |bytes| {
        push_uint_property(
            bytes,
            "KeyedProperty",
            "propertyKey",
            u64::from(property_key_for_name("Node", "x")),
        );
    });
    for (index, &(frame, value)) in keys.iter().enumerate() {
        let linear = u64::from(index + 1 < keys.len());
        push_keyframe_double(bytes, frame, value, linear);
    }
    if let Some(frame) = event_frame {
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", PING);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Event", "trigger")),
            );
        });
        push_object_with_properties(bytes, "KeyFrameCallback", |bytes| {
            push_uint_property(bytes, "KeyFrameCallback", "frame", frame);
        });
    }
}

/// One artboard with node `n` (x keyed by every animation), three events and
/// one state machine of one layer: Any, Entry (to the first authored state),
/// the authored states, Exit. The ToC lists 60023 as a uint.
fn write(file_id: u64, spec: &Spec) -> Vec<u8> {
    synthetic_runtime_file_with_toc(file_id, &[(is_hold_key(), 0)], |bytes| {
        if spec.view_model {
            push_object_with_properties(bytes, "ViewModel", |bytes| {
                push_string_property(bytes, "ViewModel", "name", "Root");
            });
            push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
                push_string_property(bytes, "ViewModelPropertyString", "name", "step");
            });
        }
        push_object_with_properties(bytes, "Backboard", |_| {});
        if spec.view_model {
            push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
                push_string_property(bytes, "ViewModelInstance", "name", "root");
                push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
            });
            push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
                push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
                push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "");
            });
        }
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Node", |bytes| {
            push_string_property(bytes, "Node", "name", "n");
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 0.0);
        });
        for name in ["ping", "enter", "start"] {
            push_object_with_properties(bytes, "Event", |bytes| {
                push_string_property(bytes, "Event", "name", name);
                push_uint_property(bytes, "Event", "parentId", 0);
            });
        }
        push_animation(bytes, "move", 60, &[(0, 0.0), (60, 100.0)], spec.move_event);
        push_animation(bytes, "glide", 60, &[(0, 200.0), (60, 300.0)], None);
        push_animation(bytes, "rest", 60, &[(0, 50.0)], None);
        push_animation(bytes, "short", 6, &[(0, 0.0), (6, 10.0)], None);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        for input in &spec.inputs {
            let (type_name, name) = match *input {
                Input::Bool(name) => ("StateMachineBool", name),
                Input::Trigger(name) => ("StateMachineTrigger", name),
                Input::Number(name) => ("StateMachineNumber", name),
            };
            push_object_with_properties(bytes, type_name, |bytes| {
                push_string_property(bytes, type_name, "name", name);
            });
        }
        push_object_with_properties(bytes, "StateMachineLayer", |bytes| {
            push_string_property(bytes, "StateMachineLayer", "name", "holds");
        });
        push_object_with_properties(bytes, "AnyState", |_| {});
        for transition in &spec.any {
            push_transition(bytes, transition);
        }
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_transition(bytes, &to(FIRST, &[]));
        for state in &spec.states {
            match state.kind {
                Kind::Anim(anim) => {
                    push_object_with_properties(bytes, "AnimationState", |bytes| {
                        push_uint_property(bytes, "AnimationState", "animationId", anim.id());
                    });
                }
                Kind::Blend1D(input) => {
                    push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
                        push_uint_property(bytes, "BlendState1DInput", "inputId", input);
                    });
                    push_blend_animation_1d(bytes, Anim::Move.id(), 0.0);
                    push_blend_animation_1d(bytes, Anim::Glide.id(), 1.0);
                }
            }
            if state.enter_event {
                push_fire_event(bytes, ENTER);
            }
            for transition in &state.transitions {
                push_transition(bytes, transition);
            }
        }
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

const A: u64 = FIRST;
const B: u64 = FIRST + 1;

/// The test 1 file: A plays `move` and goes to B (`glide`) on exit time 100%;
/// `a_extra` are A's other transitions.
fn track_file(
    file_id: u64,
    inputs: Vec<Input>,
    view_model: bool,
    a_extra: Vec<Tr>,
    move_event: Option<u64>,
) -> Vec<u8> {
    let mut a = vec![exit_at_end(B)];
    a.extend(a_extra);
    write(
        file_id,
        &Spec {
            view_model,
            inputs,
            states: vec![state(Anim::Move, a), state(Anim::Glide, vec![])],
            move_event,
            ..Spec::default()
        },
    )
}

fn import(bytes: &[u8]) -> Option<RuntimeFileHandle> {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory"),
        None,
        None,
        None,
    )
}

fn first_layer(machine: &RuntimeStateMachineInstanceHandle) -> CoreHandle {
    machine
        .with_instance(|machine| machine.state_machine())
        .with_downcast::<StateMachine, _>(|machine| machine.layer(0))
        .flatten()
        .expect("layer 0")
}

struct Run {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: Option<CoreHandle>,
}

impl Run {
    fn new(bytes: &[u8]) -> Self {
        let file = import(bytes).expect("fixture imports");
        let artboard = file
            .with_file(|file| file.artboard_default())
            .expect("artboard instance");
        let machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine instance");
        let view_model = file.with_file(|file| file.view_model(0)).and_then(|model| {
            model
                .with_downcast::<ViewModel, _>(|model| model.instance_at(0))
                .flatten()
        });
        if let Some(instance) = &view_model {
            machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
        }
        Self {
            _file: file,
            artboard,
            machine,
            view_model,
        }
    }

    fn advance(&self, seconds: f32) {
        self.machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
    }

    fn advance_and_apply(&self, seconds: f32) {
        self.machine.advance_and_apply(seconds);
    }

    fn x(&self) -> f32 {
        let node = self
            .artboard
            .with_artboard(|artboard| artboard.base.resolve_handle(NODE))
            .expect("node n");
        CoreRegistry::get_double_handle(&node, i32::from(property_key_for_name("Node", "x")))
            .expect("node x")
    }

    fn set_bool(&self, name: &str, value: bool) {
        self.machine.set_bool(name, value);
    }

    fn fire(&self, input: u32) {
        self.machine.with_instance_mut(|machine| {
            machine
                .trigger_input_mut(input)
                .expect("trigger input")
                .fire();
        });
    }

    fn set_step(&self, value: &str) {
        let instance = self.view_model.as_ref().expect("bound view model");
        let step = instance
            .with_downcast::<ViewModelInstance, _>(|instance| {
                instance
                    .property_value_named("step")
                    .or_else(|| instance.property_value_by_id(0))
            })
            .flatten()
            .expect("step property");
        step.with_downcast_mut::<ViewModelInstanceString, _>(|step| step.set_value(value))
            .expect("step is a string");
    }

    /// The index in layer 0 of its current state.
    fn state(&self) -> Option<u64> {
        let current = self
            .machine
            .with_instance_mut(|machine| machine.layer_state(0))?;
        first_layer(&self.machine)
            .with_downcast::<StateMachineLayer, _>(|layer| {
                layer.states().iter().position(|state| *state == current)
            })
            .flatten()
            .map(|index| index as u64)
    }

    fn has_state_holds(&self) -> bool {
        first_layer(&self.machine)
            .with_downcast::<StateMachineLayer, _>(StateMachineLayer::has_state_holds)
            .expect("StateMachineLayer")
    }

    /// Names of the events the last advance reported.
    fn events(&self) -> Vec<String> {
        let events = self.machine.with_instance(|machine| {
            (0..machine.reported_event_count())
                .map(|index| machine.reported_event_at(index).event)
                .collect::<Vec<_>>()
        });
        events
            .into_iter()
            .map(|event| {
                event
                    .expect("reported event has an Event")
                    .with(|event| event.as_component().expect("Event").name().to_owned())
                    .expect("live Event")
            })
            .collect()
    }

    fn reported(&self, name: &str) -> bool {
        self.events().iter().any(|event| event == name)
    }

    fn changed_states(&self) -> usize {
        self.machine
            .with_instance(|machine| machine.state_changed_count())
    }

    fn frozen(&self) -> bool {
        self.machine
            .with_instance(|machine| machine.is_layer_frozen(0))
    }

    fn quiet_state(&self) -> QuietState {
        self.machine
            .with_instance(|machine| machine.row_quiet_state())
    }
}

fn held_layer_advances() -> u64 {
    StateMachineInstance::held_layer_advances()
}

/// Advances the held build by `DT` and the twin by zero seconds.
fn held_advance(held: &Run, twin: &Run) {
    held.advance(DT);
    twin.advance(0.0);
}

fn assert_matches_twin(held: &Run, twin: &Run, label: &str) {
    assert_eq!(held.x(), twin.x(), "{label}: x");
    assert_eq!(held.state(), twin.state(), "{label}: current state");
    assert_eq!(held.events(), twin.events(), "{label}: reported events");
}

// Copied from cpp_probe_transitions_native.rs (MachineAction and the probe
// argument loop of compare_native_machine_actions), for the two actions test
// 11 uses.
#[derive(Clone, Copy)]
enum MachineAction {
    SetBool(u32, bool),
    Advance(f32),
}

fn probe_machine_actions(
    probe: &Path,
    label: &str,
    bytes: &[u8],
    actions: &[MachineAction],
) -> CppProbeFile {
    let mut args = Vec::new();
    for action in actions {
        match action {
            MachineAction::SetBool(index, value) => args.extend([
                "--runtime-set-state-machine-bool".to_owned(),
                "0".to_owned(),
                index.to_string(),
                value.to_string(),
            ]),
            MachineAction::Advance(seconds) => args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]),
        }
    }
    read_cpp_probe_bytes_with_args(probe, label, bytes, &args)
}

#[test]
fn a_hold_stands_the_layer_still_and_carries_on_from_the_same_point() {
    let bytes = track_file(
        9101,
        vec![Input::Bool("hold")],
        false,
        vec![hold(A, &[Cond::Bool(0)])],
        None,
    );
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    let plain = Run::new(&bytes);
    let mut first_b = [None; 3];
    let mut index = 0;
    let mut note = |index: usize| {
        for (first, run) in first_b.iter_mut().zip([&held, &twin, &plain]) {
            if first.is_none() && run.state() == Some(B) {
                *first = Some(index);
            }
        }
    };

    // Entering A from Entry spends no time, so A is at 14/60 s after 15.
    for _ in 0..15 {
        index += 1;
        for run in [&held, &twin, &plain] {
            run.advance(DT);
        }
        note(index);
    }
    let x0 = held.x();
    held.set_bool("hold", true);
    let before = held_layer_advances();
    index += 1;
    held_advance(&held, &twin);
    plain.advance(DT);
    note(index);
    assert_eq!(held.x(), x0, "the first held advance writes nothing");
    assert_eq!(held_layer_advances(), before + 1);
    for _ in 0..30 {
        index += 1;
        held_advance(&held, &twin);
        plain.advance(DT);
        note(index);
        assert_eq!(held.x(), x0, "a held advance writes nothing");
        assert_eq!(held.state(), Some(A));
    }
    held.set_bool("hold", false);
    for _ in 0..90 {
        index += 1;
        for run in [&held, &twin, &plain] {
            run.advance(DT);
        }
        note(index);
        assert_matches_twin(&held, &twin, &format!("advance {index}"));
    }
    let [held_b, twin_b, plain_b] = first_b;
    assert!(plain_b.is_some(), "the plain run reaches B");
    assert_eq!(held_b, twin_b, "A to B on the twin's advance");
    assert_eq!(
        held_b,
        plain_b.map(|index| index + 31),
        "late by the 31 held advances"
    );
}

#[test]
fn a_view_model_hold_stands_a_playing_track_on_its_first_advance() {
    let bytes = track_file(9102, vec![], true, vec![hold(A, &[Cond::Step("s1")])], None);
    let run = Run::new(&bytes);
    for _ in 0..15 {
        run.advance(DT);
    }
    let x0 = run.x();
    assert!(x0 > 0.0, "move plays");
    run.set_step("s1");
    run.advance(DT);
    assert_eq!(
        run.x(),
        x0,
        "the write and the hold land in the same advance"
    );
}

#[test]
fn a_hold_on_the_state_a_mix_runs_from_stops_the_mix() {
    let bytes = write(
        9103,
        &Spec {
            inputs: vec![
                Input::Bool("go"),
                Input::Bool("holdA"),
                Input::Bool("holdB"),
            ],
            states: vec![
                state(
                    Anim::Move,
                    vec![
                        Tr {
                            to: B,
                            duration: 400,
                            conditions: vec![Cond::Bool(0)],
                            ..Tr::default()
                        },
                        hold(A, &[Cond::Bool(1)]),
                    ],
                ),
                state(Anim::Glide, vec![hold(B, &[Cond::Bool(2)])]),
            ],
            ..Spec::default()
        },
    );
    for (input, label) in [
        ("holdA", "hold on the from-state"),
        ("holdB", "hold on the current state"),
    ] {
        let held = Run::new(&bytes);
        let twin = Run::new(&bytes);
        for _ in 0..15 {
            held.advance(DT);
            twin.advance(DT);
        }
        held.set_bool("go", true);
        twin.set_bool("go", true);
        held.advance(DT);
        twin.advance(DT);
        assert_eq!(held.state(), Some(B), "{label}: A to B starts its mix");
        // 12 advances of a 400 ms mix: mix 0.5.
        for _ in 0..12 {
            held.advance(DT);
            twin.advance(DT);
        }
        let x0 = held.x();
        held.set_bool(input, true);
        for advance in 0..30 {
            held_advance(&held, &twin);
            assert_eq!(held.x(), x0, "{label}: held advance {advance}");
        }
        held.set_bool(input, false);
        for advance in 0..60 {
            held.advance(DT);
            twin.advance(DT);
            assert_matches_twin(
                &held,
                &twin,
                &format!("{label}: released advance {advance}"),
            );
        }
    }
}

#[test]
fn a_layer_held_mid_mix_still_takes_transitions() {
    const C: u64 = FIRST + 2;
    let bytes = write(
        9104,
        &Spec {
            inputs: vec![Input::Bool("toB"), Input::Bool("holdA"), Input::Bool("go")],
            states: vec![
                state(
                    Anim::Move,
                    vec![
                        // Early exit off.
                        Tr {
                            to: B,
                            duration: 400,
                            conditions: vec![Cond::Bool(0)],
                            ..Tr::default()
                        },
                        hold(A, &[Cond::Bool(1)]),
                    ],
                ),
                state(Anim::Glide, vec![to(C, &[Cond::Bool(2)])]),
                state(Anim::Rest, vec![]),
            ],
            ..Spec::default()
        },
    );
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    for _ in 0..15 {
        held.advance(DT);
        twin.advance(DT);
    }
    held.set_bool("toB", true);
    twin.set_bool("toB", true);
    held.advance(DT);
    twin.advance(DT);
    assert_eq!(held.state(), Some(B));
    // 6 advances of a 400 ms mix: mix 0.25.
    for _ in 0..6 {
        held.advance(DT);
        twin.advance(DT);
    }
    let x0 = held.x();
    held.set_bool("holdA", true);
    held_advance(&held, &twin);
    assert_eq!(held.x(), x0, "held on the from-state");
    held.set_bool("go", true);
    twin.set_bool("go", true);
    held_advance(&held, &twin);
    assert_eq!(held.state(), Some(C), "a held layer searches mid-mix");
    assert_eq!(held.x(), x0, "and writes nothing");
    // Positive control: upstream's guard keeps the twin in B until its mix
    // ends, 18 advances from mix 0.25.
    assert_eq!(twin.state(), Some(B), "the twin keeps the early-exit guard");
    for advance in 0..17 {
        twin.advance(DT);
        assert_eq!(twin.state(), Some(B), "twin advance {advance} is mid-mix");
    }
    let mut reached = false;
    for _ in 0..3 {
        twin.advance(DT);
        if twin.state() == Some(C) {
            reached = true;
            break;
        }
    }
    assert!(reached, "the twin takes B to C once its mix ends");
}

#[test]
fn an_any_transition_taken_while_held_moves_nothing() {
    const C: u64 = FIRST + 1;
    let bytes = write(
        9105,
        &Spec {
            inputs: vec![Input::Bool("holdA"), Input::Bool("toC")],
            any: vec![Tr {
                to: C,
                duration: 300,
                conditions: vec![Cond::Bool(1)],
                ..Tr::default()
            }],
            states: vec![
                state(Anim::Move, vec![hold(A, &[Cond::Bool(0)])]),
                state(Anim::Glide, vec![]),
            ],
            ..Spec::default()
        },
    );
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    for _ in 0..15 {
        held.advance(DT);
        twin.advance(DT);
    }
    let x0 = held.x();
    held.set_bool("holdA", true);
    held_advance(&held, &twin);
    assert_eq!(held.x(), x0);
    held.set_bool("toC", true);
    twin.set_bool("toC", true);
    held_advance(&held, &twin);
    assert_eq!(held.state(), Some(C), "Any to C is taken while held");
    assert_eq!(held.x(), x0, "and writes nothing");
    for advance in 0..10 {
        held_advance(&held, &twin);
        assert_eq!(held.x(), x0, "A's hold holds the mix: advance {advance}");
        assert_eq!(held.state(), Some(C));
    }
    held.set_bool("holdA", false);
    for advance in 0..40 {
        held.advance(DT);
        twin.advance(DT);
        assert_matches_twin(&held, &twin, &format!("released advance {advance}"));
    }
}

#[test]
fn a_trigger_fired_while_held_moves_the_held_layer() {
    const D: u64 = FIRST + 1;
    let trigger_file = |file_id: u64, hold_conditions: &[Cond], mix: bool| {
        let to_d = if mix {
            Tr {
                to: D,
                flags: ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE | PAUSE_ON_EXIT,
                exit_time: Some(0),
                duration: 200,
                conditions: vec![Cond::Trigger(1)],
                start_event: true,
                ..Tr::default()
            }
        } else {
            to(D, &[Cond::Trigger(1)])
        };
        write(
            file_id,
            &Spec {
                inputs: vec![Input::Bool("holdA"), Input::Trigger("T")],
                states: vec![
                    state(Anim::Move, vec![to_d, hold(A, hold_conditions)]),
                    St {
                        kind: Kind::Anim(Anim::Glide),
                        transitions: vec![],
                        enter_event: mix,
                    },
                ],
                ..Spec::default()
            },
        )
    };

    // The take: A to D on T, exit time 0%, pause on exit, a 200 ms mix.
    let bytes = trigger_file(9106, &[Cond::Bool(0)], true);
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    for _ in 0..15 {
        held.advance(DT);
        twin.advance(DT);
    }
    let x0 = held.x();
    let before = held_layer_advances();
    held.set_bool("holdA", true);
    held.fire(1);
    twin.fire(1);
    held_advance(&held, &twin);
    assert_eq!(held_layer_advances(), before + 1, "the advance is held");
    assert_eq!(held.state(), Some(D), "T moves the held layer");
    assert_eq!(held.x(), x0, "and writes nothing");
    let events = held.events();
    assert_eq!(events, twin.events(), "events report as on the twin");
    assert!(
        events.iter().any(|event| event == "enter") && events.iter().any(|event| event == "start"),
        "D's entry event and the transition's start event: {events:?}"
    );
    for advance in 0..5 {
        held_advance(&held, &twin);
        assert_eq!(held.x(), x0, "A's hold holds the mix: advance {advance}");
    }
    held.set_bool("holdA", false);
    for advance in 0..40 {
        held.advance(DT);
        twin.advance(DT);
        assert_matches_twin(&held, &twin, &format!("released advance {advance}"));
    }

    // T also among the hold's conditions: the hold never consumes it.
    let bytes = trigger_file(9107, &[Cond::Bool(0), Cond::Trigger(1)], true);
    let run = Run::new(&bytes);
    for _ in 0..15 {
        run.advance(DT);
    }
    let before = held_layer_advances();
    run.set_bool("holdA", true);
    run.fire(1);
    run.advance(DT);
    assert_eq!(
        held_layer_advances(),
        before + 1,
        "the hold reads T as fired"
    );
    assert_eq!(run.state(), Some(D), "a hold never consumes a trigger");

    // No mix, D plays glide and has no hold: the advance after the take moves.
    let bytes = trigger_file(9108, &[Cond::Bool(0)], false);
    let run = Run::new(&bytes);
    for _ in 0..15 {
        run.advance(DT);
    }
    let x0 = run.x();
    run.set_bool("holdA", true);
    run.advance(DT);
    assert_eq!(run.x(), x0);
    run.fire(1);
    run.advance(DT);
    assert_eq!(run.state(), Some(D));
    assert_eq!(run.x(), x0, "a held advance writes nothing, even the take");
    run.advance(DT);
    assert_ne!(run.x(), x0, "D has no hold: x moves");
}

#[test]
fn entering_a_held_state_writes_nothing() {
    // P0 rests at x = 50; one write makes P0 to P1 true and P1's hold pass.
    const P0: u64 = FIRST;
    const P1: u64 = FIRST + 1;
    let bytes = write(
        9109,
        &Spec {
            inputs: vec![Input::Bool("go")],
            states: vec![
                state(Anim::Rest, vec![to(P1, &[Cond::Bool(0)])]),
                state(Anim::Move, vec![hold(P1, &[Cond::Bool(0)])]),
            ],
            ..Spec::default()
        },
    );
    let run = Run::new(&bytes);
    for _ in 0..5 {
        run.advance(DT);
    }
    assert_eq!(run.state(), Some(P0));
    assert_eq!(run.x(), 50.0);
    run.set_bool("go", true);
    run.advance(DT);
    assert_eq!(run.state(), Some(P1));
    assert_eq!(run.x(), 50.0, "P1 is entered held and writes nothing");
    for advance in 0..5 {
        run.advance(DT);
        assert_eq!(run.x(), 50.0, "held advance {advance}");
    }

    // R plays short and goes to P at its end; the third 0.075 s advance
    // crosses R's end with 0.05 s to spill into P, which is held.
    const R: u64 = FIRST;
    const P: u64 = FIRST + 1;
    let bytes = write(
        9110,
        &Spec {
            inputs: vec![Input::Bool("holdP")],
            states: vec![
                state(Anim::Short, vec![exit_at_end(P)]),
                state(Anim::Move, vec![hold(P, &[Cond::Bool(0)])]),
            ],
            move_event: Some(1),
            ..Spec::default()
        },
    );
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    held.set_bool("holdP", true);
    held.advance(0.075);
    twin.advance(0.075);
    assert_eq!(held.state(), Some(R), "the first advance enters R at 0");
    for _ in 0..2 {
        held.advance(0.075);
        twin.advance(0.075);
    }
    assert_eq!(held.state(), Some(P));
    assert!(!held.reported("ping"), "P's clock has not moved");
    assert_eq!(twin.state(), Some(P));
    assert!(twin.reported("ping"), "the twin's P took the spill");
    held.set_bool("holdP", false);
    held.advance(DT);
    twin.advance(DT);
    assert!(
        held.reported("ping"),
        "P's event reports on the first released advance"
    );
    assert_eq!(held.x(), twin.x());
    assert_eq!(held.state(), twin.state());
    for advance in 0..20 {
        held.advance(DT);
        twin.advance(DT);
        assert_eq!(held.x(), twin.x(), "released advance {advance}");
        assert_eq!(held.state(), twin.state(), "released advance {advance}");
    }
}

#[test]
fn a_change_out_of_a_state_entered_held_writes_nothing_more() {
    // Q0 rests at x = 50. One write takes Q0 to Q1 (move), whose hold
    // passes, and on through Q1's zero-duration way out to Q2 (glide, from
    // x = 200), which has no hold, in one ordinary advance.
    const Q0: u64 = FIRST;
    const Q1: u64 = FIRST + 1;
    const Q2: u64 = FIRST + 2;
    let chain_file = |file_id: u64, with_hold: bool| {
        let mut q1 = vec![to(Q2, &[Cond::Bool(0)])];
        if with_hold {
            q1.push(hold(Q1, &[Cond::Bool(0)]));
        }
        write(
            file_id,
            &Spec {
                inputs: vec![Input::Bool("go")],
                states: vec![
                    state(Anim::Rest, vec![to(Q1, &[Cond::Bool(0)])]),
                    state(Anim::Move, q1),
                    state(Anim::Glide, vec![]),
                ],
                ..Spec::default()
            },
        )
    };
    let held = Run::new(&chain_file(9114, true));
    let plain = Run::new(&chain_file(9115, false));
    for _ in 0..5 {
        held.advance(DT);
        plain.advance(DT);
    }
    assert_eq!(held.state(), Some(Q0));
    assert_eq!(held.x(), 50.0);
    held.set_bool("go", true);
    plain.set_bool("go", true);
    held.advance(DT);
    plain.advance(DT);
    assert_eq!(plain.state(), Some(Q2), "positive control: Q0 to Q1 to Q2");
    assert_eq!(plain.x(), 200.0, "positive control: Q2 writes at 0");
    assert_eq!(
        held.state(),
        Some(Q2),
        "Q1's way out is taken in the advance"
    );
    assert_eq!(
        held.x(),
        50.0,
        "Q1 was entered held: nothing more is written, not even Q2"
    );
    for advance in 0..10 {
        held.advance(DT);
        plain.advance(DT);
        assert_eq!(held.x(), plain.x(), "released advance {advance}");
        assert_eq!(held.state(), plain.state(), "released advance {advance}");
    }
    assert_ne!(held.x(), 50.0, "Q2 plays once released");
}

#[test]
fn a_self_transition_without_the_property_changes_nothing() {
    let inputs = || vec![Input::Bool("hold")];
    let self_transition = track_file(9081, inputs(), false, vec![to(A, &[Cond::Bool(0)])], None);
    let disabled_hold = track_file(
        9082,
        inputs(),
        false,
        vec![Tr {
            flags: DISABLED,
            ..hold(A, &[Cond::Bool(0)])
        }],
        None,
    );
    let without = track_file(9083, inputs(), false, vec![], None);
    for (bytes, label) in [
        (&self_transition, "a self-transition"),
        (&disabled_hold, "a disabled hold"),
    ] {
        let run = Run::new(bytes);
        let reference = Run::new(&without);
        run.set_bool("hold", true);
        reference.set_bool("hold", true);
        let mut last = None;
        for advance in 1..=130 {
            run.advance(DT);
            reference.advance(DT);
            assert_eq!(run.x(), reference.x(), "{label}: advance {advance}");
            assert_eq!(run.state(), reference.state(), "{label}: advance {advance}");
            if (2..=59).contains(&advance) {
                assert_ne!(Some(run.x()), last, "{label}: x moves at advance {advance}");
                assert_eq!(
                    run.changed_states(),
                    0,
                    "{label}: A is not replaced at advance {advance}"
                );
            }
            last = Some(run.x());
        }
    }
}

#[test]
fn a_finished_one_shot_settles_and_freezes_again_after_release() {
    let bytes = write(
        9091,
        &Spec {
            view_model: true,
            states: vec![state(
                Anim::Move,
                vec![hold(A, &[Cond::Step("s1")]), hold(A, &[Cond::Step("s2")])],
            )],
            ..Spec::default()
        },
    );
    let run = Run::new(&bytes);
    let skips = StateMachineInstance::settled_layer_skips();
    for _ in 0..90 {
        run.advance_and_apply(DT);
    }
    assert!(run.frozen(), "move finished: the layer settles and freezes");
    assert!(StateMachineInstance::settled_layer_skips() > skips);
    run.set_step("s2");
    for advance in 0..3 {
        let before = held_layer_advances();
        run.advance_and_apply(DT);
        assert!(
            !run.frozen(),
            "a held layer is never frozen: advance {advance}"
        );
        assert!(
            held_layer_advances() > before,
            "held through s2: advance {advance}"
        );
        assert_eq!(run.quiet_state(), QuietState::Busy, "advance {advance}");
    }
    run.set_step("");
    let mut frozen = false;
    for _ in 0..2 {
        run.advance_and_apply(DT);
        if run.frozen() {
            frozen = true;
            break;
        }
    }
    assert!(frozen, "settled and frozen again within two advances");
}

#[test]
fn a_keyed_event_inside_the_held_span_reports_after_release() {
    // ping is keyed at 0.5 s of move.
    let bytes = track_file(
        9100,
        vec![Input::Bool("hold")],
        false,
        vec![hold(A, &[Cond::Bool(0)])],
        Some(30),
    );
    let held = Run::new(&bytes);
    let twin = Run::new(&bytes);
    for _ in 0..16 {
        held.advance(DT);
        twin.advance(DT);
        assert!(!held.reported("ping"));
    }
    held.set_bool("hold", true);
    for advance in 0..30 {
        held_advance(&held, &twin);
        assert!(
            !held.reported("ping"),
            "no report while held: advance {advance}"
        );
    }
    held.set_bool("hold", false);
    let (mut held_at, mut twin_at) = (None, None);
    for advance in 0..40 {
        held.advance(DT);
        twin.advance(DT);
        assert_eq!(held.events(), twin.events(), "released advance {advance}");
        if held_at.is_none() && held.reported("ping") {
            held_at = Some(advance);
        }
        if twin_at.is_none() && twin.reported("ping") {
            twin_at = Some(advance);
        }
    }
    assert!(
        held_at.is_some(),
        "ping reports once track time crosses 0.5 s"
    );
    assert_eq!(held_at, twin_at, "on the twin's advance");
}

#[test]
fn an_old_reader_skips_the_hold_and_plays_through() {
    let probe = probe_path();
    let actions = [
        MachineAction::Advance(0.0),
        MachineAction::SetBool(0, true),
        MachineAction::Advance(0.25),
    ];
    let hold_bytes = synthetic_state_machine_same_state_hold(9111);
    let plain_bytes = synthetic_state_machine_same_state_transition(9111);
    let hold_report = probe_machine_actions(
        &probe,
        "synthetic/state_hold_same_state_cpp.riv",
        &hold_bytes,
        &actions,
    );
    let plain_report = probe_machine_actions(
        &probe,
        "synthetic/state_hold_same_state_plain_cpp.riv",
        &plain_bytes,
        &actions,
    );
    assert_eq!(
        hold_report, plain_report,
        "the pinned reader sees an inert transition to the current state"
    );
    let node = hold_report
        .artboards
        .first()
        .and_then(|artboard| artboard.runtime_update.as_ref())
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == NODE as usize)
        })
        .and_then(|component| component.local_transform)
        .expect("the node's local transform");
    assert!(
        (node[4] - 2.0).abs() > 0.1,
        "the pinned reader plays through: {node:?}"
    );

    // Positive control: Nuxie's reader holds the state.
    let run = Run::new(&hold_bytes);
    run.advance(0.0);
    let x0 = run.x();
    run.set_bool("go", true);
    run.advance(0.25);
    assert_eq!(run.x(), x0, "the hold stands the node still");
}

#[test]
fn a_hold_missing_from_the_toc_fails_an_old_reader() {
    let probe = probe_path();
    let with_toc = synthetic_state_machine_same_state_hold(9112);
    let without_toc = synthetic_state_machine_same_state_hold_with_toc(9112, &[]);
    let (status, stderr) = cpp_probe_import_status(
        &probe,
        "synthetic/state_hold_without_toc_cpp.riv",
        &without_toc,
    );
    assert!(!status.success(), "the pinned reader refuses the file");
    assert!(
        stderr.contains("Unknown property key 60023, missing from property ToC."),
        "{stderr}"
    );
    assert!(stderr.contains("result=malformed"), "{stderr}");
    assert!(import(&with_toc).is_some(), "Nuxie's reader, ToC listed");
    assert!(
        import(&without_toc).is_some(),
        "Nuxie's reader knows 60023 without the ToC"
    );
    // Any object carrying 60023 is read as a bool without the ToC.
    let node = synthetic_runtime_file_with_toc(9113, &[], |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_var_uint(bytes, u64::from(is_hold_key()));
            bytes.push(1);
        });
    });
    assert!(import(&node).is_some(), "a Node carrying 60023 imports");
}

#[test]
fn holds_that_break_a_rule_refuse_the_file() {
    let rule_file = |file_id: u64, any: Vec<Tr>, a_extra: Vec<Tr>| {
        write(
            file_id,
            &Spec {
                inputs: vec![Input::Bool("hold")],
                any,
                states: vec![state(Anim::Move, a_extra), state(Anim::Glide, vec![])],
                ..Spec::default()
            },
        )
    };
    let refused = [
        (
            "a hold whose target is another state",
            rule_file(9131, vec![], vec![hold(B, &[Cond::Bool(0)])]),
        ),
        (
            "a hold on the Any state",
            rule_file(9132, vec![hold(ANY, &[Cond::Bool(0)])], vec![]),
        ),
        (
            "a hold that enables exit time",
            rule_file(
                9133,
                vec![],
                vec![Tr {
                    flags: ENABLE_EXIT_TIME,
                    ..hold(A, &[Cond::Bool(0)])
                }],
            ),
        ),
        (
            "a hold with no condition",
            rule_file(9134, vec![], vec![hold(A, &[])]),
        ),
    ];
    for (rule, bytes) in &refused {
        assert!(import(bytes).is_none(), "{rule} refuses the file");
    }

    // Positive controls: a hold that obeys the rules, as type 65 and as
    // type 78 from a blend state.
    let plain = Run::new(&rule_file(9135, vec![], vec![hold(A, &[Cond::Bool(0)])]));
    assert!(plain.has_state_holds(), "a StateTransition hold registers");
    let blend = Run::new(&write(
        9136,
        &Spec {
            inputs: vec![Input::Bool("hold"), Input::Number("blend")],
            states: vec![St {
                kind: Kind::Blend1D(1),
                transitions: vec![Tr {
                    blend: true,
                    ..hold(A, &[Cond::Bool(0)])
                }],
                enter_event: false,
            }],
            ..Spec::default()
        },
    ));
    assert!(
        blend.has_state_holds(),
        "a BlendStateTransition hold registers"
    );
    for _ in 0..15 {
        blend.advance(DT);
    }
    let x0 = blend.x();
    assert!(x0 > 0.0, "the blend state plays move");
    blend.set_bool("hold", true);
    blend.advance(DT);
    assert_eq!(blend.x(), x0, "one held advance on the blend state");
}

#[test]
fn a_file_without_holds_never_checks_one() {
    let self_transition = track_file(
        9141,
        vec![Input::Bool("hold")],
        false,
        vec![to(A, &[Cond::Bool(0)])],
        None,
    );
    let run = Run::new(&self_transition);
    run.set_bool("hold", true);
    for _ in 0..120 {
        run.advance(DT);
    }
    assert_eq!(StateMachineInstance::hold_checks(), 0, "a self-transition");

    let run = Run::new(&synthetic_state_machine_same_state_transition(9142));
    run.set_bool("go", true);
    for _ in 0..120 {
        run.advance(DT);
    }
    assert_eq!(
        StateMachineInstance::hold_checks(),
        0,
        "the same-state fixture"
    );

    let file = import(&live_move_fixture::fixture(2)).expect("live-move fixture imports");
    let root = file
        .with_file(|file| file.artboard_named("Page"))
        .expect("Page");
    let model = file
        .with_file(|file| file.create_default_view_model_instance_for_artboard(root.core_handle()))
        .expect("Page view model");
    let machine = root.default_state_machine_handle().expect("Page machine");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    for _ in 0..120 {
        machine.advance_and_apply(DT);
    }
    assert_eq!(
        StateMachineInstance::hold_checks(),
        0,
        "the live-move fixture"
    );
}
