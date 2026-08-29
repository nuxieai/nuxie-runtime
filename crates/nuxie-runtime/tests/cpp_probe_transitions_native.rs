//! Transition differentials over the pinned source owners.
//!
//! The probe's --runtime-advance-state-machine calls StateMachineInstance::advance,
//! not advanceAndApply. Each observation below is captured immediately after that
//! call; only the final explicit update settles component transforms. See pinned
//! src/animation/state_machine_instance.cpp and src/animation/state_transition.cpp.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, RecordingFactory};
use nuxie_runtime::source::{
    animation::animation_state::AnimationState,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard as NativeArtboard, RuntimeArtboardInstanceHandle},
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::{File as NativeFile, RuntimeFileHandle},
    generated::core_registry::CoreRegistry as NativeCoreRegistry,
    math::random::RandomProvider,
};
use serde::Deserialize;

mod cpp_probe_support;
use cpp_probe_support::*;
#[path = "cpp_probe_support/transition_fixtures.rs"]
mod transition_fixtures;
use transition_fixtures::*;

struct RuntimeRandomTestValuesGuard;

impl Drop for RuntimeRandomTestValuesGuard {
    fn drop(&mut self) {
        RandomProvider::clear_testing_mode();
    }
}

fn set_runtime_random_test_values(values: &[f32]) -> RuntimeRandomTestValuesGuard {
    RandomProvider::clear_randoms();
    for value in values {
        RandomProvider::add_random_value(*value);
    }
    RuntimeRandomTestValuesGuard
}

fn runtime_random_call_count() -> usize {
    RandomProvider::total_calls() as usize
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}
#[derive(Debug, Deserialize)]
struct CppArtboard {
    #[serde(rename = "runtimeStateMachineAdvances", default)]
    runtime_state_machine_advances: Vec<CppRuntimeStateMachineAdvance>,
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
}

// Wire/observation values only, never a second mutable execution graph.
#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineAdvance {
    #[serde(rename = "stateMachineIndex")]
    state_machine_index: usize,
    advanced: bool,
    #[serde(rename = "currentAnimationCount")]
    current_animation_count: usize,
    #[serde(rename = "changedStateCount")]
    changed_state_count: usize,
    #[serde(rename = "changedStateCoreTypes", default)]
    changed_state_core_types: Vec<u16>,
    #[serde(rename = "reportedEventCount", default)]
    reported_event_count: usize,
    #[serde(rename = "currentAnimations")]
    current_animations: Vec<CppRuntimeStateMachineCurrentAnimation>,
    #[serde(rename = "reportedEvents", default)]
    reported_events: Vec<CppRuntimeStateMachineReportedEvent>,
    #[serde(rename = "viewModelTriggers", default)]
    view_model_triggers: Vec<CppRuntimeStateMachineViewModelTrigger>,
    #[serde(rename = "randomTotalCalls", default)]
    random_total_calls: usize,
}

fn set_bool(machine: &RuntimeStateMachineInstanceHandle, index: u32, value: bool) -> bool {
    let name = machine.with_instance(|machine| {
        machine
            .bool_input(index)
            .map(|input| input.base.name().to_owned())
    });
    let Some(name) = name else {
        return false;
    };
    machine.set_bool(&name, value);
    true
}

fn set_number(machine: &RuntimeStateMachineInstanceHandle, index: u32, value: f32) -> bool {
    let name = machine.with_instance(|machine| {
        machine
            .number_input(index)
            .map(|input| input.base.name().to_owned())
    });
    let Some(name) = name else {
        return false;
    };
    machine.set_number(&name, value);
    true
}

fn fire_trigger(machine: &RuntimeStateMachineInstanceHandle, index: u32) -> bool {
    machine.with_instance_mut(|machine| {
        let Some(input) = machine.trigger_input_mut(index) else {
            return false;
        };
        input.fire();
        true
    })
}

fn transform_x(artboard: &RuntimeArtboardInstanceHandle, local_id: usize) -> f32 {
    native_double(artboard, local_id, property_key_for_name("Node", "x"))
}

fn observe_state_machine(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
) -> CppRuntimeStateMachineAdvance {
    let definition = machine.with_instance(|machine| machine.state_machine());
    let state_machine_index = artboard.with_artboard(|artboard| {
        (0..artboard.base.state_machine_count())
            .position(|index| {
                artboard.base.state_machine_handle_at(index).as_ref() == Some(&definition)
            })
            .expect("state machine belongs to this artboard")
    });
    let (
        current_animation_count,
        current_animations,
        changed_state_count,
        changed_state_core_types,
        reported_event_count,
        events,
    ) = machine.with_instance_mut(|machine| {
        let current_animation_count = machine.current_animation_count();
        let current_animations = (0..current_animation_count)
            .map(|index| {
                machine
                    .current_animation_by_index(index)
                    .expect("current animation occurrence")
                    .first_animation(|animation| CppRuntimeStateMachineCurrentAnimation {
                        time: animation.time(),
                        did_loop: animation.did_loop(),
                    })
                    .expect("AnimationState owns its LinearAnimationInstance")
            })
            .collect();
        let changed_state_count = machine.state_changed_count();
        let changed_state_core_types = (0..changed_state_count)
            .map(|index| {
                machine
                    .state_changed_by_index(index)
                    .expect("changed state")
                    .core_type()
                    .expect("live changed state")
            })
            .collect();
        let reported_event_count = machine.reported_event_count();
        let events = (0..reported_event_count)
            .map(|index| {
                let report = machine.reported_event_at(index);
                (
                    report.event.expect("reported event has a live Event owner"),
                    report.seconds_delay,
                )
            })
            .collect::<Vec<_>>();
        (
            current_animation_count,
            current_animations,
            changed_state_count,
            changed_state_core_types,
            reported_event_count,
            events,
        )
    });
    let reported_events = events
        .into_iter()
        .map(|(event, seconds_delay)| {
            let local = artboard.with_artboard(|artboard| artboard.base.object_index(&event));
            CppRuntimeStateMachineReportedEvent {
                event_local: usize::try_from(local).ok(),
                event_core_type: event.core_type().map(u32::from),
                event_name: event.with(|event| {
                    event
                        .as_component()
                        .expect("Event component")
                        .name()
                        .to_owned()
                }),
                seconds_delay,
            }
        })
        .collect();
    // Match collect_default_view_model_trigger_reports in the C++ probe: these
    // are the authored first model/instance values, not a cloned SMI context.
    let instance = file.with_file(|file| file.view_model(0)).and_then(|model| {
        model
            .with(|model| model.as_view_model().expect("ViewModel").instance_at(0))
            .flatten()
    });
    let values = instance
        .map(|instance| {
            instance
                .with(|instance| {
                    instance
                        .as_view_model_instance()
                        .expect("ViewModelInstance")
                        .property_values()
                        .to_vec()
                })
                .expect("live instance")
        })
        .unwrap_or_default();
    let mut view_model_triggers = Vec::new();
    for value in values {
        let trigger = value
            .with(|value| {
                value.as_view_model_instance_trigger().map(|trigger| {
                    (
                        trigger.base.view_model_property_id(),
                        u64::from(trigger.base.property_value()),
                    )
                })
            })
            .flatten();
        if let Some((view_model_property_id, value)) = trigger {
            view_model_triggers.push(CppRuntimeStateMachineViewModelTrigger {
                index: view_model_triggers.len(),
                view_model_property_id,
                value,
            });
        }
    }
    CppRuntimeStateMachineAdvance {
        state_machine_index,
        advanced,
        current_animation_count,
        current_animations,
        changed_state_count,
        changed_state_core_types,
        reported_event_count,
        reported_events,
        view_model_triggers,
        random_total_calls: runtime_random_call_count(),
    }
}

fn compare_state_machine_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &CppRuntimeStateMachineAdvance,
    advanced: bool,
    label: &str,
) {
    assert_eq!(
        cpp.state_machine_index, rust.state_machine_index,
        "{label} stateMachineIndex mismatch"
    );
    assert_eq!(cpp.advanced, advanced, "{label} advance return mismatch");
    assert_eq!(
        rust.advanced, advanced,
        "{label} captured advance return mismatch"
    );
    assert_eq!(
        cpp.current_animation_count, rust.current_animation_count,
        "{label} currentAnimationCount mismatch"
    );
    assert_eq!(
        cpp.changed_state_count, rust.changed_state_count,
        "{label} changedStateCount mismatch"
    );
    assert_eq!(
        cpp.changed_state_core_types, rust.changed_state_core_types,
        "{label} stateChangedByIndex current-state order mismatch"
    );
    assert_eq!(
        cpp.reported_event_count, rust.reported_event_count,
        "{label} reportedEventCount mismatch"
    );
    assert_eq!(
        cpp.current_animations.len(),
        rust.current_animations.len(),
        "{label} current animation observations"
    );
    for (index, cpp_animation) in cpp.current_animations.iter().enumerate() {
        let rust_animation = &rust.current_animations[index];
        assert_close(
            cpp_animation.time,
            rust_animation.time,
            &format!("{label} current animation {index} time"),
        );
        assert_eq!(
            cpp_animation.did_loop, rust_animation.did_loop,
            "{label} current animation {index} didLoop mismatch"
        );
    }
    assert_eq!(
        cpp.reported_events.len(),
        rust.reported_events.len(),
        "{label} reported event observations"
    );
    for (index, cpp_event) in cpp.reported_events.iter().enumerate() {
        let rust_event = &rust.reported_events[index];
        assert_eq!(
            cpp_event.event_local, rust_event.event_local,
            "{label} reported event {index} local ID mismatch"
        );
        assert_eq!(
            cpp_event.event_core_type, rust_event.event_core_type,
            "{label} reported event {index} core type mismatch"
        );
        assert_eq!(
            cpp_event.event_name, rust_event.event_name,
            "{label} reported event {index} name mismatch"
        );
        assert_close(
            cpp_event.seconds_delay,
            rust_event.seconds_delay,
            &format!("{label} reported event {index} secondsDelay"),
        );
    }
    assert_eq!(
        cpp.view_model_triggers.len(),
        rust.view_model_triggers.len(),
        "{label} viewModelTriggers count mismatch"
    );
    for (index, cpp_trigger) in cpp.view_model_triggers.iter().enumerate() {
        let rust_trigger = &rust.view_model_triggers[index];
        assert_eq!(
            cpp_trigger.index, index,
            "{label} view-model trigger {index} index mismatch"
        );
        assert_eq!(
            rust_trigger.index, index,
            "{label} native view-model trigger {index} index mismatch"
        );
        assert_eq!(
            cpp_trigger.view_model_property_id, rust_trigger.view_model_property_id,
            "{label} view-model trigger {index} property ID mismatch"
        );
        assert_eq!(
            cpp_trigger.value, rust_trigger.value,
            "{label} view-model trigger {index} value mismatch"
        );
    }
}

#[derive(Debug, Deserialize)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "graphOrder")]
    graph_order: Option<usize>,
    scheduled: bool,
    dirt: u16,
    collapsed: bool,
    #[serde(rename = "worldTransform")]
    world_transform: Option<[f32; 6]>,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
    #[serde(rename = "renderOpacity")]
    render_opacity: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeUpdate {
    #[serde(rename = "didUpdate")]
    did_update: bool,
    #[serde(rename = "hasComponentsDirt")]
    has_components_dirt: bool,
    components: Vec<CppRuntimeComponent>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineCurrentAnimation {
    time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineReportedEvent {
    #[serde(rename = "eventLocal")]
    event_local: Option<usize>,
    #[serde(rename = "eventCoreType")]
    event_core_type: Option<u32>,
    #[serde(rename = "eventName")]
    event_name: Option<String>,
    #[serde(rename = "secondsDelay")]
    seconds_delay: f32,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineViewModelTrigger {
    index: usize,
    #[serde(rename = "viewModelPropertyId")]
    view_model_property_id: u32,
    value: u64,
}

fn read_native_instance_from_bytes(
    bytes: &[u8],
    label: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let mut factory = nuxie_render_api::PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory");
    let file = NativeFile::import(bytes, factory, None, None, None)
        .unwrap_or_else(|| panic!("failed to import {label}"));
    let instance = file
        .with_file(|file| file.artboard_default())
        .unwrap_or_else(|| panic!("failed to instantiate native artboard for {label}"));
    (file, instance)
}

fn native_object(instance: &RuntimeArtboardInstanceHandle, local_id: usize) -> CoreHandle {
    instance
        .with_artboard(|artboard| artboard.base.resolve_handle(local_id as u32))
        .unwrap_or_else(|| panic!("missing native object {local_id}"))
}

fn native_double(instance: &RuntimeArtboardInstanceHandle, local_id: usize, key: u16) -> f32 {
    NativeCoreRegistry::get_double_handle(&native_object(instance, local_id), i32::from(key))
        .unwrap_or_else(|| panic!("missing native property {local_id}:{key}"))
}

fn compare_native_runtime_update(
    cpp: &CppProbeFile,
    rust: &RuntimeArtboardInstanceHandle,
    did_update: bool,
    label: &str,
) {
    let cpp_update = cpp
        .artboards
        .first()
        .and_then(|artboard| artboard.runtime_update.as_ref())
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));
    assert_eq!(cpp_update.did_update, did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.with_artboard(|artboard| artboard.base.has_component_dirt())
    );
    for cpp_component in &cpp_update.components {
        compare_native_component(cpp_component, rust, label);
    }
}

fn compare_native_component(
    cpp: &CppRuntimeComponent,
    artboard: &RuntimeArtboardInstanceHandle,
    label: &str,
) {
    let object = native_object(artboard, cpp.local_id);
    let scheduled = artboard.with_artboard(|artboard| {
        artboard
            .base
            .dependency_order()
            .iter()
            .any(|component| component.authored() == Some(&object))
    });
    let (graph_order, dirt, collapsed, local, world, opacity) = object
        .with(|object| {
            let component = object.as_component().expect("native Component");
            (
                scheduled.then_some(component.graph_order() as usize),
                component.dirt().0,
                object
                    .as_layout_component()
                    .map(|layout| layout.is_collapsed())
                    .unwrap_or_else(|| component.is_collapsed()),
                object
                    .as_transform_component()
                    .map(|transform| Mat2D(*transform.transform().values())),
                object
                    .as_world_transform_component()
                    .map(|transform| Mat2D(*transform.world_transform().values())),
                object
                    .as_transform_component()
                    .map(|transform| transform.render_opacity()),
            )
        })
        .expect("live native Component");
    assert_eq!(
        cpp.scheduled, scheduled,
        "schedule membership mismatch for local {} in {label}",
        cpp.local_id
    );
    if cpp.scheduled {
        assert_eq!(
            cpp.graph_order, graph_order,
            "graph order mismatch for scheduled local {} in {label}",
            cpp.local_id
        );
        assert!(
            cpp.graph_order.is_some(),
            "scheduled local {} omitted graph order in {label}",
            cpp.local_id
        );
    } else {
        assert_eq!(
            None, cpp.graph_order,
            "unscheduled local {} exposed indeterminate C++ graph order in {label}",
            cpp.local_id
        );
        assert_eq!(
            None, graph_order,
            "unscheduled local {} manufactured graph order in {label}",
            cpp.local_id
        );
    }
    assert_eq!(
        cpp.dirt, dirt,
        "dirt mismatch for local {} in {label}",
        cpp.local_id
    );
    assert_eq!(
        cpp.collapsed, collapsed,
        "collapsed flag mismatch for local {} in {label}",
        cpp.local_id
    );
    compare_mat2d(
        cpp.local_transform,
        local,
        "local transform",
        cpp.local_id,
        label,
    );
    compare_mat2d(
        cpp.world_transform,
        world,
        "world transform",
        cpp.local_id,
        label,
    );
    compare_optional_f32(
        cpp.render_opacity,
        opacity,
        "render opacity",
        cpp.local_id,
        label,
    );
}

#[derive(Clone, Copy)]
enum MachineAction {
    SetBool(u32, bool),
    SetNumber(u32, f32),
    FireTrigger(u32),
    Advance(f32),
}

fn compare_native_machine_actions(
    label: &str,
    bytes: &[u8],
    actions: &[MachineAction],
) -> Vec<CppRuntimeStateMachineAdvance> {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");
    let mut args = Vec::new();
    for action in actions {
        match action {
            MachineAction::SetBool(index, value) => args.extend([
                "--runtime-set-state-machine-bool".to_owned(),
                "0".to_owned(),
                index.to_string(),
                value.to_string(),
            ]),
            MachineAction::SetNumber(index, value) => args.extend([
                "--runtime-set-state-machine-number".to_owned(),
                "0".to_owned(),
                index.to_string(),
                value.to_string(),
            ]),
            MachineAction::FireTrigger(index) => args.extend([
                "--runtime-fire-state-machine-trigger".to_owned(),
                "0".to_owned(),
                index.to_string(),
            ]),
            MachineAction::Advance(seconds) => args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]),
        }
    }

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, bytes, &args);
    let (file, rust) = read_native_instance_from_bytes(bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));
    let mut rust_reports = Vec::new();
    for action in actions {
        match *action {
            MachineAction::SetBool(index, value) => assert!(set_bool(&machine, index, value)),
            MachineAction::SetNumber(index, value) => {
                assert!(set_number(&machine, index, value))
            }
            MachineAction::FireTrigger(index) => assert!(fire_trigger(&machine, index)),
            MachineAction::Advance(seconds) => {
                let advanced =
                    machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
                rust_reports.push(observe_state_machine(&file, &rust, &machine, advanced));
            }
        }
    }
    let did_update = NativeArtboard::update_components_handle(&rust.core_handle());
    let cpp_reports = &cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"))
        .runtime_state_machine_advances;
    assert_eq!(
        cpp_reports.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_report, rust_report) in cpp_reports.iter().zip(&rust_reports) {
        compare_state_machine_advance(cpp_report, rust_report, rust_report.advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, did_update, label);
    rust_reports
}

#[test]
fn state_machine_entry_timed_transition_starts_destination_at_zero_mix() {
    let label = "synthetic/runtime_state_machine_entry_timed_transition_public.riv";
    let bytes = synthetic_state_machine_entry_timed_transition(8261);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_eq!(
        state_machine.with_instance(|machine| machine.state_changed_count()),
        1
    );
    assert_close(
        transform_x(&rust, 1),
        2.0,
        "entry transition target animation starts at mix 0",
    );

    assert!(state_machine.with_instance_mut(|machine| machine.advance_seconds(0.5)));
    assert_close(
        transform_x(&rust, 1),
        13.5,
        "entry transition target animation mixes after elapsed time",
    );
}

#[test]
fn state_machine_input_transitions_match_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (file_id, kind, label) in [
        (
            8235,
            SyntheticInputTransitionKind::Bool,
            "synthetic/runtime_state_machine_bool_transition_cpp.riv",
        ),
        (
            8236,
            SyntheticInputTransitionKind::Number,
            "synthetic/runtime_state_machine_number_transition_cpp.riv",
        ),
        (
            8237,
            SyntheticInputTransitionKind::Trigger,
            "synthetic/runtime_state_machine_trigger_transition_cpp.riv",
        ),
    ] {
        let bytes = synthetic_state_machine_input_transition(file_id, kind);
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];
        match kind {
            SyntheticInputTransitionKind::Bool => {
                args.extend([
                    "--runtime-set-state-machine-bool".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                    "true".to_owned(),
                ]);
            }
            SyntheticInputTransitionKind::Number => {
                args.extend([
                    "--runtime-set-state-machine-number".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                    "4".to_owned(),
                ]);
            }
            SyntheticInputTransitionKind::Trigger => {
                args.extend([
                    "--runtime-fire-state-machine-trigger".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                ]);
            }
        }
        args.extend([
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ]);

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        match kind {
            SyntheticInputTransitionKind::Bool => {
                assert!(set_bool(&state_machine, 0, true));
            }
            SyntheticInputTransitionKind::Number => {
                assert!(set_number(&state_machine, 0, 4.0));
            }
            SyntheticInputTransitionKind::Trigger => {
                assert!(fire_trigger(&state_machine, 0));
            }
        }
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_timed_transition_mixing_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (file_id, label, post_transition_advances) in [
        (
            8238,
            "synthetic/runtime_state_machine_timed_transition_start_cpp.riv",
            Vec::<f32>::new(),
        ),
        (
            8239,
            "synthetic/runtime_state_machine_timed_transition_half_cpp.riv",
            vec![0.5],
        ),
        (
            8240,
            "synthetic/runtime_state_machine_timed_transition_complete_cpp.riv",
            vec![0.5, 0.5],
        ),
    ] {
        let bytes = synthetic_state_machine_input_transition_with_duration(
            file_id,
            SyntheticInputTransitionKind::Bool,
            1000,
        );
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];
        for seconds in &post_transition_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        assert!(set_bool(&state_machine, 0, true));
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        for seconds in post_transition_advances {
            let advanced =
                state_machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
            rust_reports.push((
                advanced,
                observe_state_machine(&_file, &rust, &state_machine, advanced),
            ));
        }
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_exit_time_transition_matches_cpp_probe() {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;

    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_exit_time_transition_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8241,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME,
                    exit_time: Some(1000),
                    ..Default::default()
                },
            ),
            vec![0.5, 0.5],
        ),
        (
            "synthetic/runtime_state_machine_any_exit_time_transition_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8242,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME,
                    exit_time: Some(1000),
                    any_state_transition: true,
                    ..Default::default()
                },
            ),
            vec![0.0],
        ),
        (
            "synthetic/runtime_state_machine_zero_duration_loop_exit_time_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8246,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
                    exit_time: Some(0),
                    source_animation_duration: 0,
                    ..Default::default()
                },
            ),
            // Pinned C++ computes floor(0/0)*0, producing NaN. `time < NaN`
            // is false, so the transition is allowed on this same advance
            // (`state_transition.cpp:147-174`).
            vec![0.0],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        assert!(set_bool(&state_machine, 0, true));
        for seconds in post_set_advances {
            let advanced =
                state_machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
            rust_reports.push((
                advanced,
                observe_state_machine(&_file, &rust, &state_machine, advanced),
            ));
        }
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_transition_handoff_matches_cpp_probe() {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const PAUSE_ON_EXIT: u64 = 1 << 4;

    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_spilled_time_handoff_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8243,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions::default(),
            ),
            vec![2.5],
        ),
        (
            "synthetic/runtime_state_machine_pause_on_exit_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8244,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    duration: 1000,
                    flags: ENABLE_EXIT_TIME | PAUSE_ON_EXIT,
                    exit_time: Some(1000),
                    source_second_frame: 20,
                    source_second_value: 22.0,
                    ..Default::default()
                },
            ),
            vec![1.5, 0.5],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        assert!(set_bool(&state_machine, 0, true));
        for seconds in post_set_advances {
            let advanced =
                state_machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
            rust_reports.push((
                advanced,
                observe_state_machine(&_file, &rust, &state_machine, advanced),
            ));
        }
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_percentage_timing_matches_cpp_probe() {
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;

    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_percentage_duration_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8245,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    duration: 50,
                    flags: DURATION_IS_PERCENTAGE,
                    ..Default::default()
                },
            ),
            vec![0.0, 0.5, 0.5],
        ),
        (
            "synthetic/runtime_state_machine_percentage_exit_time_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8246,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
                    exit_time: Some(50),
                    ..Default::default()
                },
            ),
            vec![0.5, 0.5],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        assert!(set_bool(&state_machine, 0, true));
        for seconds in post_set_advances {
            let advanced =
                state_machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
            rust_reports.push((
                advanced,
                observe_state_machine(&_file, &rust, &state_machine, advanced),
            ));
        }
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_cubic_transition_interpolator_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let label = "synthetic/runtime_state_machine_cubic_transition_interpolator_cpp.riv";
    let bytes = synthetic_state_machine_input_transition_with_options(
        8247,
        SyntheticInputTransitionKind::Bool,
        SyntheticTransitionOptions {
            duration: 1000,
            cubic_transition_interpolator: true,
            ..Default::default()
        },
    );
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 0, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.5));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_elastic_transition_interpolator_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let label = "synthetic/runtime_state_machine_elastic_transition_interpolator_cpp.riv";
    let bytes = synthetic_state_machine_input_transition_with_options(
        8248,
        SyntheticInputTransitionKind::Bool,
        SyntheticTransitionOptions {
            duration: 1000,
            elastic_transition_interpolator: true,
            ..Default::default()
        },
    );
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 0, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.5));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_early_exit_transition_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let label = "synthetic/runtime_state_machine_early_exit_transition_cpp.riv";
    let bytes = synthetic_state_machine_early_exit_transition(8249);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 0, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 1, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.5));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_blend_state_early_exit_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let label = "synthetic/runtime_state_machine_blend_state_early_exit_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_early_exit(8264);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "2".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(set_number(&state_machine, 0, 0.5));
    let mut rust_reports = Vec::new();
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 1, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 2, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.5));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_random_transition_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let label = "synthetic/runtime_state_machine_random_transition_cpp.riv";
    let bytes = synthetic_state_machine_random_transition(8250);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let state_machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    assert!(set_bool(&state_machine, 0, true));
    let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    rust_reports.push((
        advanced,
        observe_state_machine(&_file, &rust, &state_machine, advanced),
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_blend_state_random_transition_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    for (label, bytes, number_value) in [
        (
            "synthetic/runtime_state_machine_blend_state_1d_random_transition_cpp.riv",
            synthetic_state_machine_blend_state_random_transition(
                8265,
                SyntheticRandomBlendSource::Blend1D,
            ),
            0.5,
        ),
        (
            "synthetic/runtime_state_machine_blend_state_direct_random_transition_cpp.riv",
            synthetic_state_machine_blend_state_random_transition(
                8266,
                SyntheticRandomBlendSource::Direct,
            ),
            50.0,
        ),
    ] {
        let args = [
            "--runtime-set-state-machine-number".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            number_value.to_string(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
            "true".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0.5".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0.5".to_owned(),
        ];

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let state_machine = rust
            .state_machine_instance_handle(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        assert!(set_number(&state_machine, 0, number_value));
        let mut rust_reports = Vec::new();
        let advanced = state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        rust_reports.push((
            advanced,
            observe_state_machine(&_file, &rust, &state_machine, advanced),
        ));
        assert!(set_bool(&state_machine, 1, true));
        for seconds in [0.0, 0.5, 0.5] {
            let advanced =
                state_machine.with_instance_mut(|machine| machine.advance_seconds(seconds));
            rust_reports.push((
                advanced,
                observe_state_machine(&_file, &rust, &state_machine, advanced),
            ));
        }
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn state_machine_direct_blend_state_transition_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_direct_blend_state_transition_cpp.riv";
    let bytes = synthetic_state_machine_direct_blend_state_transition(8260);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 50.0),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_blend_state_transition_exit_time_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_transition_exit_time_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_transition(8258);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_blend_state_transition_reset_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_transition_reset_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_transition_reset(8259);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_blend_state_percentage_duration_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_percentage_duration_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_percentage_duration(8261);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_blend_state_percentage_exit_time_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_percentage_exit_time_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_percentage_exit_time(8262);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(1.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_blend_state_pause_on_exit_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_pause_on_exit_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_pause_on_exit(8263);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_animation_state_advance_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_animation_state.riv";
    let bytes = synthetic_state_machine_animation_state(8231);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
            MachineAction::Advance(0.0),
        ],
    );
}

#[test]
fn state_machine_animation_state_advances_through_public_runtime_seam() {
    let label = "synthetic/runtime_state_machine_animation_state_public.riv";
    let bytes = synthetic_state_machine_animation_state(8230);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    assert_eq!(
        rust.with_artboard(|artboard| artboard.base.state_machine_count()),
        1
    );
    let machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_eq!(
        machine.with_instance_mut(|machine| machine.current_animation_count()),
        1
    );
    assert_close(
        machine
            .with_instance_mut(|machine| machine.current_animation_by_index(0))
            .and_then(|state| state.first_animation(|animation| animation.time()))
            .expect("entered animation state"),
        0.0,
        "entered animation state time",
    );
    assert_close(transform_x(&rust, 1), 2.0, "entered animation state x");

    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.5)));
    assert_close(
        machine
            .with_instance_mut(|machine| machine.current_animation_by_index(0))
            .and_then(|state| state.first_animation(|animation| animation.time()))
            .expect("advanced animation state"),
        0.5,
        "advanced animation state time",
    );
    assert_close(transform_x(&rust, 1), 7.0, "advanced animation state x");
    assert!(
        !machine.with_instance_mut(|machine| machine.advance_seconds(0.0)),
        "zero-second state-machine advance after entering an animation state matches C++ cached keep-going behavior"
    );
}

#[test]
fn state_machine_bool_input_drives_zero_duration_transition() {
    let label = "synthetic/runtime_state_machine_bool_transition_public.riv";
    let bytes = synthetic_state_machine_input_transition(8232, SyntheticInputTransitionKind::Bool);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));
    machine.with_instance(|machine| {
        assert_eq!(machine.input_count(), 1);
        let input = machine.bool_input(0).expect("named bool input");
        assert_eq!(input.base.name(), "armed");
        assert!(!input.value());
    });
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_close(
        transform_x(&rust, 1),
        2.0,
        "initial bool transition state x",
    );
    assert!(set_bool(&machine, 0, true));
    assert!(
        machine.with_instance(|machine| machine.bool_input(0).is_some_and(|input| input.value()))
    );
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_eq!(
        machine.with_instance(|machine| machine.state_changed_count()),
        1
    );
    assert_close(
        transform_x(&rust, 1),
        20.0,
        "bool transition target state x",
    );
}

#[test]
fn state_machine_number_input_drives_zero_duration_transition() {
    let label = "synthetic/runtime_state_machine_number_transition_public.riv";
    let bytes =
        synthetic_state_machine_input_transition(8233, SyntheticInputTransitionKind::Number);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));
    machine.with_instance(|machine| {
        let input = machine.number_input(0).expect("named number input");
        assert_eq!(input.base.name(), "level");
        assert_eq!(input.value(), 0.0);
    });
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_close(
        transform_x(&rust, 1),
        2.0,
        "initial number transition state x",
    );
    assert!(set_number(&machine, 0, 4.0));
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_eq!(
        machine.with_instance(|machine| machine.state_changed_count()),
        1
    );
    assert_close(
        transform_x(&rust, 1),
        20.0,
        "number transition target state x",
    );
}

#[test]
fn state_machine_trigger_input_drives_zero_duration_transition_once() {
    let label = "synthetic/runtime_state_machine_trigger_transition_public.riv";
    let bytes =
        synthetic_state_machine_input_transition(8234, SyntheticInputTransitionKind::Trigger);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));
    machine.with_instance(|machine| {
        let input = machine.trigger_input(0).expect("named trigger input");
        assert_eq!(input.base.name(), "go");
        assert!(!input.fired());
    });
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_close(
        transform_x(&rust, 1),
        2.0,
        "initial trigger transition state x",
    );
    assert!(fire_trigger(&machine, 0));
    assert!(
        machine
            .with_instance(|machine| machine.trigger_input(0).is_some_and(|input| input.fired()))
    );
    assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
    assert_eq!(
        machine.with_instance(|machine| machine.state_changed_count()),
        1
    );
    assert!(
        machine
            .with_instance(|machine| machine.trigger_input(0).is_some_and(|input| !input.fired()))
    );
    assert_close(
        transform_x(&rust, 1),
        20.0,
        "trigger transition target state x",
    );
}

#[test]
fn state_machine_input_conditions_reject_wrong_types_and_bad_indices_like_cpp() {
    for (offset, (case, input_kind, condition_kind, input_id)) in [
        (
            "bool-wrong-type",
            SyntheticInputTransitionKind::Number,
            SyntheticInputTransitionKind::Bool,
            0,
        ),
        (
            "bool-bad-index",
            SyntheticInputTransitionKind::Bool,
            SyntheticInputTransitionKind::Bool,
            1,
        ),
        (
            "number-wrong-type",
            SyntheticInputTransitionKind::Trigger,
            SyntheticInputTransitionKind::Number,
            0,
        ),
        (
            "number-bad-index",
            SyntheticInputTransitionKind::Number,
            SyntheticInputTransitionKind::Number,
            1,
        ),
        (
            "trigger-wrong-type",
            SyntheticInputTransitionKind::Bool,
            SyntheticInputTransitionKind::Trigger,
            0,
        ),
        (
            "trigger-bad-index",
            SyntheticInputTransitionKind::Trigger,
            SyntheticInputTransitionKind::Trigger,
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let label = format!("synthetic/runtime_state_machine_input_condition_{case}.riv");
        let bytes = synthetic_state_machine_input_transition_with_condition(
            8800 + offset as u64,
            input_kind,
            condition_kind,
            input_id,
            SyntheticTransitionOptions::default(),
        );
        compare_native_machine_actions(&label, &bytes, &[MachineAction::Advance(0.0)]);
        let (_file, rust) = read_native_instance_from_bytes(&bytes, &label);
        let machine = rust
            .state_machine_instance_handle(0)
            .expect("native state machine");
        machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        NativeArtboard::update_components_handle(&rust.core_handle());
        assert_close(
            transform_x(&rust, 1),
            20.0,
            &format!("{label} rejected condition"),
        );
    }
}

#[test]
fn state_machine_input_conditions_preserve_cpp_null_slots_and_evaluate_them_true() {
    for (offset, kind) in [
        SyntheticInputTransitionKind::Bool,
        SyntheticInputTransitionKind::Number,
        SyntheticInputTransitionKind::Trigger,
    ]
    .into_iter()
    .enumerate()
    {
        let label = format!("synthetic/runtime_state_machine_null_{kind:?}_condition.riv");
        let bytes = synthetic_state_machine_input_transition_with_condition_and_null_slot(
            8810 + offset as u64,
            kind,
            kind,
            0,
            SyntheticTransitionOptions::default(),
            true,
        );
        let (_file, rust) = read_native_instance_from_bytes(&bytes, &label);
        let machine = rust
            .state_machine_instance_handle(0)
            .expect("native state machine");
        machine.with_instance(|machine| {
            assert_eq!(machine.input_count(), 2, "{label} compacted the null slot");
            assert!(machine.input(0).is_none(), "{label} exposed a null input");
            assert!(
                machine.input(1).is_some(),
                "{label} shifted the concrete input"
            );
        });
        assert!(machine.with_instance_mut(|machine| machine.advance_seconds(0.0)));
        NativeArtboard::update_components_handle(&rust.core_handle());
        assert_close(
            transform_x(&rust, 1),
            20.0,
            &format!("{label} null condition"),
        );
    }
}

#[test]
fn fl_c5_state_changed_layers_and_convergence_match_cpp_probe() {
    let label = "synthetic/fl_c5_state_changed_layers_and_convergence.riv";
    let bytes = synthetic_fl_c5_layer_state_queries(90_508);
    let reports = compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::Advance(0.0),
            MachineAction::SetBool(0, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.0),
        ],
    );
    assert_eq!(reports[0].changed_state_count, 3);
    assert_eq!(reports[1].changed_state_count, 2);
    assert_eq!(reports[1].changed_state_core_types.len(), 2);
    assert_eq!(reports[2].changed_state_count, 0);
}

#[test]
fn state_machine_last_authored_entry_state_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_duplicate_entry_cpp.riv";
    let bytes = synthetic_state_machine_duplicate_system_states(8273);
    compare_native_machine_actions(label, &bytes, &[MachineAction::Advance(0.0)]);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .expect("native state machine");
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    let current = machine
        .with_instance_mut(|machine| machine.current_animation_by_index(0))
        .expect("current animation state");
    let expected = rust
        .with_artboard(|artboard| artboard.base.animation_handle_at(1))
        .expect("second authored animation");
    assert_eq!(
        current
            .definition()
            .with_downcast::<AnimationState, _>(AnimationState::animation)
            .flatten(),
        Some(expected),
        "the last authored EntryState must select the second animation"
    );
}

#[test]
fn state_machine_generic_layer_state_occurrence_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_generic_layer_state_cpp.riv";
    let bytes = synthetic_state_machine_generic_system_state(8278);
    compare_native_machine_actions(
        label,
        &bytes,
        &[MachineAction::Advance(0.0), MachineAction::Advance(0.25)],
    );
}

#[test]
fn fl_c5_current_state_and_animation_authored_compression_match_cpp_probe() {
    let label = "synthetic/fl_c5_current_state_and_animation.riv";
    let bytes = synthetic_fl_c5_layer_state_queries(90_509);
    let reports = compare_native_machine_actions(label, &bytes, &[MachineAction::Advance(0.0)]);
    assert_eq!(reports[0].current_animation_count, 2);
    assert_eq!(reports[0].changed_state_count, 3);
    assert_eq!(reports[0].changed_state_core_types.len(), 3);
}

#[test]
fn state_machine_transition_interruption_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_transition_interruption_cpp.riv";
    let bytes = synthetic_state_machine_transition_interruption(8276);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::Advance(0.0),
            MachineAction::SetBool(0, true),
            MachineAction::Advance(0.0),
            MachineAction::SetBool(1, true),
            MachineAction::Advance(0.0),
            MachineAction::Advance(0.5),
        ],
    );
}

#[test]
fn state_machine_same_state_transition_is_a_noop_like_cpp_probe() {
    let label = "synthetic/runtime_state_machine_same_state_transition_cpp.riv";
    let bytes = synthetic_state_machine_same_state_transition(8277);
    let reports = compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::Advance(0.0),
            MachineAction::SetBool(0, true),
            MachineAction::Advance(0.25),
        ],
    );
    assert_eq!(
        reports[1].changed_state_count, 0,
        "a self-target must not replace the current occurrence"
    );
}

#[test]
fn state_machine_layer_entry_effects_match_cpp_after_serial_initialization() {
    let label = "synthetic/runtime_state_machine_serial_layer_entry_cpp.riv";
    let bytes = synthetic_state_machine_serial_layer_entry_initialization(8282);
    compare_native_machine_actions(label, &bytes, &[MachineAction::Advance(0.0)]);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .expect("native state machine");
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    assert_eq!(
        machine.with_instance(|machine| machine.number_input(0).map(|input| input.value())),
        Some(7.0),
        "the first authored layer's entry action runs during initialization"
    );
    assert_eq!(
        machine.with_instance_mut(|machine| machine.current_animation_count()),
        1,
        "the second layer consumes the initialized input on the first advance"
    );
}

fn counted_runtime_random_probe_args(values: &[f32], extra_args: &[String]) -> Vec<String> {
    let mut args = vec!["--runtime-random-reset".to_owned()];
    for value in values {
        args.push("--runtime-random-value".to_owned());
        args.push(value.to_string());
    }
    args.extend_from_slice(extra_args);
    args
}

#[test]
fn fl_c5_random_transition_edges_weighted_boundaries_and_wraparound_match_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");
    for (label, bytes, draw, expected_animation, expected_calls) in [
        (
            "synthetic/runtime_state_machine_weighted_random_zero_draw_cpp.riv",
            synthetic_state_machine_weighted_random_transition(8278, 1, 3),
            0.0_f32,
            1_usize,
            1_usize,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_negative_draw_cpp.riv",
            synthetic_state_machine_weighted_random_transition(82_780, 1, 3),
            -0.25_f32,
            1,
            1,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_strict_boundary_cpp.riv",
            synthetic_state_machine_weighted_random_transition(8279, 1, 3),
            0.25_f32,
            2,
            1,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_later_candidate_cpp.riv",
            synthetic_state_machine_weighted_random_transition(8280, 1, 3),
            0.75_f32,
            2,
            1,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_one_draw_selects_none_cpp.riv",
            synthetic_state_machine_weighted_random_transition(8281, 1, 3),
            1.0_f32,
            0,
            1,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_nan_selects_none_cpp.riv",
            synthetic_state_machine_weighted_random_transition(82_810, 1, 3),
            f32::NAN,
            0,
            1,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_zero_total_cpp.riv",
            synthetic_state_machine_weighted_random_transition(8282, 0, 0),
            0.75_f32,
            0,
            0,
        ),
        (
            "synthetic/runtime_state_machine_weighted_random_u32_wrap_cpp.riv",
            synthetic_state_machine_weighted_random_transition(
                8283,
                u64::from(u32::MAX),
                u64::from(u32::MAX),
            ),
            0.75_f32,
            1,
            1,
        ),
    ] {
        let args = counted_runtime_random_probe_args(
            &[draw],
            &[
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                "0".to_owned(),
            ],
        );
        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (file, rust) = read_native_instance_from_bytes(&bytes, label);
        let machine = rust
            .state_machine_instance_handle(0)
            .expect("native state machine");
        let _random_values = set_runtime_random_test_values(&[draw]);
        let advanced = machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        let report = observe_state_machine(&file, &rust, &machine, advanced);
        let cpp_report = &cpp.artboards[0].runtime_state_machine_advances[0];
        compare_state_machine_advance(cpp_report, &report, advanced, label);
        assert_eq!(
            cpp_report.random_total_calls, expected_calls,
            "{label} C++ random draw count"
        );
        assert_eq!(
            runtime_random_call_count(),
            expected_calls,
            "{label} Rust random draw count"
        );
        let current = machine
            .with_instance_mut(|machine| machine.current_animation_by_index(0))
            .expect("current animation");
        let expected = rust
            .with_artboard(|artboard| artboard.base.animation_handle_at(expected_animation))
            .expect("expected animation");
        assert_eq!(
            current
                .definition()
                .with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten(),
            Some(expected),
            "{label} selected animation"
        );
    }
}

#[test]
fn state_machine_weighted_random_wait_then_selected_transition_matches_cpp_probe() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");
    let label = "synthetic/runtime_state_machine_weighted_random_wait_then_select_cpp.riv";
    let bytes = synthetic_state_machine_weighted_random_wait_then_select(8284);
    let args = counted_runtime_random_probe_args(
        &[0.0],
        &[
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    );
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .expect("native state machine");
    let _random_values = set_runtime_random_test_values(&[0.0]);
    let advanced = machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    let report = observe_state_machine(&file, &rust, &machine, advanced);
    compare_state_machine_advance(
        &cpp.artboards[0].runtime_state_machine_advances[0],
        &report,
        advanced,
        label,
    );
    let current = machine
        .with_instance_mut(|machine| machine.current_animation_by_index(0))
        .expect("current animation");
    let expected = rust
        .with_artboard(|artboard| artboard.base.animation_handle_at(2))
        .expect("third animation");
    assert_eq!(
        current
            .definition()
            .with_downcast::<AnimationState, _>(AnimationState::animation)
            .flatten(),
        Some(expected),
        "{label} later selectable candidate"
    );
}

#[test]
fn blend_states_retain_from_to_occurrences_across_same_owner_advances_like_cpp_probe() {
    for (case, kind, input) in [
        (
            "blend-1d-same-owner",
            SyntheticCrossArtboardBlendKind::Blend1D,
            0.5,
        ),
        (
            "blend-direct-same-owner",
            SyntheticCrossArtboardBlendKind::BlendDirect,
            50.0,
        ),
    ] {
        let label = format!("synthetic/runtime_cross_artboard_{case}_definition_owner_cpp.riv");
        let bytes = synthetic_cross_artboard_blend_definition_owner(
            82_834,
            kind,
            [(2.0, 12.0), (20.0, 30.0)],
        );
        compare_native_machine_actions(
            &label,
            &bytes,
            &[
                MachineAction::SetNumber(0, input),
                MachineAction::Advance(0.0),
                MachineAction::Advance(1.0),
            ],
        );
    }
}

#[test]
fn state_machine_blend_state_1d_input_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_1d_input_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_1d_input(8256);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 0.5),
            MachineAction::Advance(0.0),
            MachineAction::Advance(1.0),
        ],
    );
}

#[test]
fn empty_baseline_animation_reset_matches_cpp_reader_underflow_zero() {
    let label = "synthetic/runtime_state_machine_empty_baseline_reset_cpp.riv";
    let bytes = synthetic_state_machine_empty_baseline_reset(8222);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 1.0),
            MachineAction::Advance(0.0),
        ],
    );
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .expect("native state machine");
    assert!(set_number(&machine, 0, 1.0));
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    NativeArtboard::update_components_handle(&rust.core_handle());
    assert_eq!(
        transform_x(&rust, 1),
        0.0,
        "empty baseline resets x to zero"
    );
}

#[test]
fn state_machine_blend_state_direct_matches_cpp_probe() {
    let label = "synthetic/runtime_state_machine_blend_state_direct_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_direct(8257);
    compare_native_machine_actions(
        label,
        &bytes,
        &[
            MachineAction::SetNumber(0, 50.0),
            MachineAction::Advance(0.0),
            MachineAction::Advance(1.0),
        ],
    );
}

#[test]
fn direct_blend_nan_mix_value_skips_target_like_cpp_probe() {
    let label = "synthetic/runtime_state_machine_direct_nan_mix_cpp.riv";
    let bytes = synthetic_state_machine_direct_nan_mix(8221);
    compare_native_machine_actions(label, &bytes, &[MachineAction::Advance(0.0)]);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let machine = rust
        .state_machine_instance_handle(0)
        .expect("native state machine");
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    NativeArtboard::update_components_handle(&rust.core_handle());
    assert_eq!(
        transform_x(&rust, 1),
        7.0,
        "ordered max/min collapses NaN to zero"
    );
}

#[test]
fn state_machine_missing_bindable_blend_instances_match_cpp_probe() {
    for (label, bytes) in [
        (
            "synthetic/runtime_state_machine_blend_state_1d_missing_bindable_instance_cpp.riv",
            synthetic_state_machine_blend_state_1d_missing_bindable_instance(8998),
        ),
        (
            "synthetic/runtime_state_machine_direct_missing_bindable_instance_cpp.riv",
            synthetic_state_machine_direct_missing_bindable_instance(8999),
        ),
    ] {
        compare_native_machine_actions(
            label,
            &bytes,
            &[MachineAction::Advance(0.0), MachineAction::Advance(1.0)],
        );
    }
}
