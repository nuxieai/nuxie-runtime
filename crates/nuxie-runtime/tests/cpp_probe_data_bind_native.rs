//! Data-bind differentials observed directly from live native owners.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine::StateMachine,
    artboard::Artboard as NativeArtboard,
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::CoreRegistry,
        data_bind::bindable_property_boolean_base::BindablePropertyBooleanBase,
        data_bind::bindable_property_number_base::BindablePropertyNumberBase,
        viewmodel::viewmodel_instance_boolean_base::ViewModelInstanceBooleanBase,
        viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    },
    math::random::RandomProvider,
    node::Node,
    viewmodel::viewmodel::ViewModel,
};
use std::sync::{LazyLock, Mutex};

static RANDOM_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};
use serde::Deserialize;

mod cpp_probe_support;
use cpp_probe_support::*;
#[path = "cpp_probe_support/transition_fixtures.rs"]
mod transition_fixtures;
use transition_fixtures::{push_animation_for_single_node, push_blend_animation_1d};

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
    view_model_triggers: Vec<serde_json::Value>,
    #[serde(rename = "numberBindings", default)]
    number_bindings: Vec<CppNumberBinding>,
    #[serde(rename = "randomTotalCalls", default)]
    random_total_calls: i32,
}

#[derive(Debug, Deserialize)]
struct CppNumberBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "targetValue")]
    target_value: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineCurrentAnimation {
    time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineReportedEvent {
    #[serde(rename = "eventCoreType")]
    event_core_type: Option<u32>,
    #[serde(rename = "eventName")]
    event_name: Option<String>,
    #[serde(rename = "secondsDelay")]
    seconds_delay: f32,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeUpdate {
    components: Vec<CppRuntimeComponent>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
}

struct NativeFixture {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    default_view_model: Option<CoreHandle>,
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    push_var_uint(
        bytes,
        u64::from(property_key_for_name(type_name, property_name)),
    );
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_bytes_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &[u8]) {
    push_var_uint(
        bytes,
        u64::from(property_key_for_name(type_name, property_name)),
    );
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn native_fixture(bytes: &[u8], label: &str) -> NativeFixture {
    native_fixture_with_default_binding(bytes, label, true)
}

fn native_fixture_with_default_binding(
    bytes: &[u8],
    label: &str,
    bind_default: bool,
) -> NativeFixture {
    let mut fixture = native_unbound_fixture(bytes, label);
    let view_model = fixture
        ._file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(fixture.artboard.core_handle())
        })
        .unwrap_or_else(|| panic!("missing native default view model for {label}"));
    if bind_default {
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
    }
    fixture.default_view_model = Some(view_model);
    fixture
}

fn native_unbound_fixture(bytes: &[u8], label: &str) -> NativeFixture {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained validation factory"),
        None,
        None,
        None,
    )
    .unwrap_or_else(|| panic!("failed to import native fixture {label}"));
    let artboard = file
        .with_file(File::artboard_default)
        .unwrap_or_else(|| panic!("missing native artboard for {label}"));
    let machine = artboard
        .state_machine_at(0)
        .unwrap_or_else(|| panic!("missing native state machine for {label}"));
    NativeFixture {
        _file: file,
        artboard,
        machine,
        default_view_model: None,
    }
}

fn bind_default_root_view_model(fixture: &NativeFixture, label: &str) -> CoreHandle {
    let view_model = fixture
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native root ViewModel");
    let instance = fixture
        ._file
        .with_file(|file| file.create_default_view_model_instance(view_model))
        .unwrap_or_else(|| panic!("missing native default root view model for {label}"));
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    instance
}

fn push_bindable_number_data_bind_context(bytes: &mut Vec<u8>, value: f32, path: &[u32]) {
    push_bindable_number_data_bind_context_with_flags(bytes, value, path, 0);
}

fn push_bindable_number_data_bind_context_with_flags(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
        push_f32_property(bytes, "BindablePropertyNumber", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyNumber",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn push_bindable_boolean_data_bind_context(bytes: &mut Vec<u8>, value: bool, path: &[u32]) {
    push_bindable_boolean_data_bind_context_with_flags(bytes, value, path, 0);
}

fn push_bindable_boolean_data_bind_context_with_flags(
    bytes: &mut Vec<u8>,
    value: bool,
    path: &[u32],
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyBoolean", |bytes| {
        push_bool_property(bytes, "BindablePropertyBoolean", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyBoolean",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn fixture_bytes(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 1.0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "alternate");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.75);
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_uint_property(bytes, "Artboard", "viewModelId", 0);
        });
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0]);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn number_two_way_fixture_bytes(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 1.0);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_flags(bytes, 0.0, &[0, 0], 1 << 1);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn nested_number_fixture_bytes(file_id: u64) -> Vec<u8> {
    nested_number_fixture_bytes_with_value(file_id, 0.0)
}

fn nested_number_fixture_bytes_with_value(file_id: u64, child_value: f32) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
            push_string_property(bytes, "ViewModelPropertyViewModel", "name", "child");
            push_uint_property(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                1,
            );
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Child");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(
                bytes,
                "ViewModelInstanceNumber",
                "propertyValue",
                child_value,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
        });
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0, 0]);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn imported_intermediate_number_fixture_bytes(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
            push_string_property(bytes, "ViewModelPropertyViewModel", "name", "child");
            push_uint_property(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                1,
            );
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Child");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.75);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
        });
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0, 0]);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn imported_intermediate_boolean_fixture_bytes(file_id: u64) -> Vec<u8> {
    nested_boolean_fixture_bytes(file_id, true)
}

fn nested_boolean_fixture_bytes(file_id: u64, child_value: bool) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
            push_string_property(bytes, "ViewModelPropertyViewModel", "name", "child");
            push_uint_property(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                1,
            );
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Child");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_bool_property(
                bytes,
                "ViewModelInstanceBoolean",
                "propertyValue",
                child_value,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
        });
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_bindable_boolean_data_bind_context(bytes, false, &[0, 0, 0]);
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueBooleanComparator", |bytes| {
            push_bool_property(bytes, "TransitionValueBooleanComparator", "value", true);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn boolean_fixture_bytes(file_id: u64) -> Vec<u8> {
    boolean_fixture_bytes_with_flags(file_id, 0)
}

fn boolean_fixture_bytes_with_flags(file_id: u64, flags: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_bool_property(bytes, "ViewModelInstanceBoolean", "propertyValue", true);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "alternate");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_bool_property(bytes, "ViewModelInstanceBoolean", "propertyValue", false);
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_uint_property(bytes, "Artboard", "viewModelId", 0);
        });
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_bindable_boolean_data_bind_context_with_flags(bytes, false, &[0, 0], flags);
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueBooleanComparator", |bytes| {
            push_bool_property(bytes, "TransitionValueBooleanComparator", "value", true);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_color_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    push_uint_property(bytes, type_name, property_name, u64::from(value));
}

fn push_bindable_number_data_bind_context_with_converter_and_flags(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    converter_id: Option<u64>,
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
        push_f32_property(bytes, "BindablePropertyNumber", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyNumber",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        if let Some(converter_id) = converter_id {
            push_uint_property(bytes, "DataBindContext", "converterId", converter_id);
        }
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn synthetic_state_machine_default_viewmodel_boolean_to_number_converter_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_boolean_to_number_converter_blend_state_with_flags(
        file_id, 0,
    )
}

fn synthetic_state_machine_default_viewmodel_boolean_to_number_converter_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_bool_property(bytes, "ViewModelInstanceBoolean", "propertyValue", true);
        });
        push_object_with_properties(bytes, "DataConverterToNumber", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_enum_to_number_converter_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_enum_to_number_converter_blend_state_with_flags(
        file_id, 0,
    )
}

fn synthetic_state_machine_default_viewmodel_enum_to_number_converter_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_string_property(bytes, "DataEnumCustom", "name", "Choice");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "First Label");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "second");
            push_string_property(bytes, "DataEnumValue", "value", "Second Label");
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "choice");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 0);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 1);
        });
        push_object_with_properties(bytes, "DataConverterToNumber", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_color_to_number_converter_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_color_to_number_converter_blend_state_with_flags(
        file_id, 0,
    )
}

fn synthetic_state_machine_default_viewmodel_color_to_number_converter_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
            push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
            push_color_property(bytes, "ViewModelInstanceColor", "propertyValue", 1);
        });
        push_object_with_properties(bytes, "DataConverterToNumber", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_string_to_number_converter_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_string_to_number_converter_blend_state_with_flags(
        file_id, 0,
    )
}

fn synthetic_state_machine_default_viewmodel_string_to_number_converter_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(
                bytes,
                "ViewModelInstanceString",
                "propertyValue",
                "1.0suffix",
            );
        });
        push_object_with_properties(bytes, "DataConverterToNumber", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_bindable_number_data_bind_context_with_converter(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    converter_id: Option<u64>,
) {
    push_bindable_number_data_bind_context_with_converter_and_flags(
        bytes,
        value,
        path,
        converter_id,
        0,
    );
}

fn cpp_random(cpp: &CppArtboard, report: usize, label: &str) -> f32 {
    let target = cpp.runtime_state_machine_advances[report]
        .number_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .and_then(|binding| binding.target_value)
        .unwrap_or_else(|| panic!("missing C++ random target for {label}"));
    (target - 2.0) / 4.0
}

fn random_formula_fixture(file_id: u64, mode: u64) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags_and_random_mode(
        file_id, 0.0, 16,
        &[FormulaFunctionArgument::Value(2.0), FormulaFunctionArgument::Value(6.0)],
        0, false, mode,
    )
}

fn assert_random_formula(label: &str, bytes: Vec<u8>, args: &[String], source_change: bool) {
    let _guard = RANDOM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(probe) = probe_path() else { return };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let board = cpp.artboards.first().expect("C++ artboard");
    let values = (0..if source_change {
        2
    } else {
        board.runtime_state_machine_advances.len()
    })
        .map(|index| cpp_random(board, index, label))
        .collect::<Vec<_>>();
    let rust = native_unbound_fixture(&bytes, label);
    let default = bind_default_root_view_model(&rust, label);
    RandomProvider::clear_randoms();
    for value in values {
        RandomProvider::add_random_value(value);
    }
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &board.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    let mut next = 1;
    if source_change {
        set_formula_number(&formula_number_property(&default), 1.0);
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(0.0));
        compare_advance(
            &board.runtime_state_machine_advances[1],
            &rust.machine,
            advanced,
            label,
        );
        next = 2;
    }
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(1.0));
    compare_advance(
        &board.runtime_state_machine_advances[next],
        &rust.machine,
        advanced,
        label,
    );
    compare_runtime_node_x(board, &rust, label);
    RandomProvider::clear_testing_mode();
}

#[test]
fn state_machine_default_viewmodel_number_formula_random_function_matches_cpp_probe() {
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_random_formula(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_cpp.riv",
        random_formula_fixture(8663, 0),
        &args,
        false,
    );
}

#[test]
fn state_machine_default_viewmodel_number_formula_random_function_always_matches_cpp_probe() {
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_random_formula(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_always_cpp.riv",
        random_formula_fixture(8804, 1),
        &args,
        false,
    );
}

#[test]
fn state_machine_default_viewmodel_number_formula_random_function_source_change_matches_cpp_probe()
{
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-default-view-model-source-number",
        "0",
        "0",
        "1",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_random_formula(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_source_change_cpp.riv",
        random_formula_fixture(8805, 2),
        &args,
        true,
    );
}

fn synthetic_state_machine_default_viewmodel_symbol_list_index_to_number_converter_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_symbol_list_index_to_number_converter_blend_state_with_flags(
        file_id, 0,
    )
}

fn synthetic_state_machine_default_viewmodel_symbol_list_index_to_number_converter_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertySymbolListIndex", |bytes| {
            push_string_property(bytes, "ViewModelPropertySymbolListIndex", "name", "symbol");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceSymbolListIndex", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "propertyValue",
                1,
            );
        });
        push_object_with_properties(bytes, "DataConverterToNumber", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_symbol_list_index_operation_value_blend_state(
    file_id: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertySymbolListIndex", |bytes| {
            push_string_property(bytes, "ViewModelPropertySymbolListIndex", "name", "symbol");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceSymbolListIndex", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "propertyValue",
                3,
            );
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_f32_property(bytes, "DataConverterOperationValue", "operationValue", 0.25);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter(bytes, 0.0, &[0, 0], Some(0));
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
    file_id: u64,
    source_value: f32,
    operation_type: u64,
    operation_value: f32,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(
                bytes,
                "ViewModelInstanceNumber",
                "propertyValue",
                source_value,
            );
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(
                bytes,
                "DataConverterOperationValue",
                "operationType",
                operation_type,
            );
            push_f32_property(
                bytes,
                "DataConverterOperationValue",
                "operationValue",
                operation_value,
            );
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter(bytes, 0.0, &[0, 0], Some(0));
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_number_formula_blend_state(
    file_id: u64,
    source_value: f32,
    operation_type: u64,
    operation_value: f32,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        file_id,
        source_value,
        operation_type,
        operation_value,
        0,
        false,
    )
}

fn synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
    file_id: u64,
    source_value: f32,
    operation_type: u64,
    operation_value: f32,
    data_bind_flags: u64,
    add_direct_observer_bind: bool,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(
                bytes,
                "ViewModelInstanceNumber",
                "propertyValue",
                source_value,
            );
        });
        push_object_with_properties(bytes, "DataConverterFormula", |_| {});
        push_object_with_properties(bytes, "FormulaTokenInput", |_| {});
        push_object_with_properties(bytes, "FormulaTokenOperation", |bytes| {
            push_uint_property(
                bytes,
                "FormulaTokenOperation",
                "operationType",
                operation_type,
            );
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_f32_property(
                bytes,
                "FormulaTokenValue",
                "operationValue",
                operation_value,
            );
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        if add_direct_observer_bind {
            push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0]);
        }
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_default_viewmodel_number_formula_group_blend_state_with_flags(
    file_id: u64,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.75);
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_f32_property(bytes, "DataConverterOperationValue", "operationValue", 2.0);
        });
        push_object_with_properties(bytes, "DataConverterFormula", |_| {});
        push_object_with_properties(bytes, "FormulaTokenInput", |_| {});
        push_object_with_properties(bytes, "FormulaTokenOperation", |bytes| {
            push_uint_property(bytes, "FormulaTokenOperation", "operationType", 2);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_f32_property(bytes, "FormulaTokenValue", "operationValue", 0.5);
        });
        push_object_with_properties(bytes, "DataConverterGroup", |_| {});
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(2),
            data_bind_flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

enum FormulaFunctionArgument {
    Input,
    Value(f32),
}

fn synthetic_state_machine_default_viewmodel_number_formula_function_blend_state(
    file_id: u64,
    source_value: f32,
    function_type: u64,
    arguments: &[FormulaFunctionArgument],
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags(
        file_id,
        source_value,
        function_type,
        arguments,
        0,
        false,
    )
}

fn synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags(
    file_id: u64,
    source_value: f32,
    function_type: u64,
    arguments: &[FormulaFunctionArgument],
    data_bind_flags: u64,
    add_direct_observer_bind: bool,
) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags_and_random_mode(
        file_id,
        source_value,
        function_type,
        arguments,
        data_bind_flags,
        add_direct_observer_bind,
        0,
    )
}

fn synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags_and_random_mode(
    file_id: u64,
    source_value: f32,
    function_type: u64,
    arguments: &[FormulaFunctionArgument],
    data_bind_flags: u64,
    add_direct_observer_bind: bool,
    random_mode_value: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(
                bytes,
                "ViewModelInstanceNumber",
                "propertyValue",
                source_value,
            );
        });
        push_object_with_properties(bytes, "DataConverterFormula", |bytes| {
            if random_mode_value != 0 {
                push_uint_property(
                    bytes,
                    "DataConverterFormula",
                    "randomModeValue",
                    random_mode_value,
                );
            }
        });
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", function_type);
        });
        for (index, argument) in arguments.iter().enumerate() {
            if index != 0 {
                push_object_with_properties(bytes, "FormulaTokenArgumentSeparator", |_| {});
            }
            match argument {
                FormulaFunctionArgument::Input => {
                    push_object_with_properties(bytes, "FormulaTokenInput", |_| {});
                }
                FormulaFunctionArgument::Value(value) => {
                    push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
                        push_f32_property(bytes, "FormulaTokenValue", "operationValue", *value);
                    });
                }
            }
        }
        push_object_with_properties(bytes, "FormulaTokenParenthesisClose", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(0),
            data_bind_flags,
        );
        if add_direct_observer_bind {
            push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0]);
        }
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn compare_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    label: &str,
) {
    assert_eq!(cpp.state_machine_index, 0, "{label} stateMachineIndex");
    assert_eq!(cpp.advanced, advanced, "{label} advance result");
    rust.with_instance_mut(|rust| {
        assert_eq!(cpp.current_animation_count, rust.current_animation_count());
        assert_eq!(cpp.changed_state_count, rust.state_changed_count());
        let changed_types = (0..rust.state_changed_count())
            .map(|index| {
                rust.state_changed_by_index(index)
                    .and_then(|state| state.core_type())
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();
        assert_eq!(cpp.changed_state_core_types, changed_types);
        assert_eq!(cpp.reported_event_count, rust.reported_event_count());
        for (index, cpp_animation) in cpp.current_animations.iter().enumerate() {
            rust.current_animation_by_index(index)
                .expect("native current animation")
                .first_animation(|animation| {
                    assert_close(animation.time(), cpp_animation.time, label);
                    assert_eq!(animation.did_loop(), cpp_animation.did_loop);
                })
                .expect("native animation state");
        }
        for (index, cpp_event) in cpp.reported_events.iter().enumerate() {
            let report = rust.reported_event_at(index);
            let event = report.event.expect("native reported event");
            assert_eq!(cpp_event.event_core_type, event.core_type().map(u32::from));
            let name = event
                .with(|event| event.as_component().map(|event| event.name().to_owned()))
                .flatten();
            assert_eq!(cpp_event.event_name.as_deref(), name.as_deref());
            assert_close(report.seconds_delay, cpp_event.seconds_delay, label);
        }
    });
    assert!(cpp.view_model_triggers.is_empty());
}

fn owned_number_context(rust: &NativeFixture, nested: bool) -> (CoreHandle, CoreHandle) {
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native root ViewModel");
    let owned = ViewModel::create_instance_handle(&model).expect("owned ViewModelInstance");
    let owner = if nested {
        owned
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .expect("root ViewModelInstance")
                    .property_value_named("child")
            })
            .flatten()
            .and_then(|child| {
                child
                    .with(|child| {
                        child
                            .as_view_model_instance_view_model()
                            .expect("ViewModelInstanceViewModel")
                            .reference_view_model_instance()
                    })
                    .flatten()
            })
            .expect("generated child ViewModelInstance")
    } else {
        owned.clone()
    };
    let number = owner
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("ViewModelInstance")
                .property_value_named("amount")
                .or_else(|| {
                    instance
                        .as_view_model_instance()
                        .and_then(|instance| instance.property_values().first().cloned())
                })
        })
        .flatten()
        .expect("owned number property");
    (owned, number)
}

fn assert_owned_number_bind(
    probe: &std::path::Path,
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    nested: bool,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, &bytes, args);
    let rust = native_unbound_fixture(&bytes, label);
    let (owned, number) = owned_number_context(&rust, nested);
    let value = 0.25_f32;
    assert!(CoreRegistry::set_double_handle(
        &number,
        i32::from(ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
        value,
    ));
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .filter_map(|component| component.local_transform.map(|transform| transform[4]))
                .last()
        })
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| {
            node.with(|node| node.as_node().map(|node| node.x()))
                .flatten()
        })
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

fn owned_context_with_imported_child(rust: &NativeFixture) -> CoreHandle {
    let root_model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native root ViewModel");
    let child_model = rust
        ._file
        .with_file(|file| file.view_model(1))
        .expect("native child ViewModel");
    let imported_child = child_model
        .with(|model| {
            model
                .as_view_model()
                .expect("child ViewModel")
                .instance_at(0)
        })
        .flatten()
        .expect("imported child ViewModelInstance");
    let owned =
        ViewModel::create_instance_handle(&root_model).expect("owned root ViewModelInstance");
    let child_property = owned
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("root ViewModelInstance")
                .property_value_named("child")
        })
        .flatten()
        .expect("owned child reference property");
    child_property
        .with_mut(|property| {
            property
                .as_view_model_instance_view_model_mut()
                .expect("ViewModelInstanceViewModel")
                .set_reference_view_model_instance(Some(imported_child));
        })
        .expect("live child reference property");
    owned
}

fn compare_runtime_node_x(cpp_artboard: &CppArtboard, rust: &NativeFixture, label: &str) {
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .filter_map(|component| component.local_transform.map(|transform| transform[4]))
                .last()
        })
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| {
            node.with(|node| node.as_node().map(|node| node.x()))
                .flatten()
        })
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

fn assert_owned_nested_boolean_bind(
    probe: &std::path::Path,
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, &bytes, args);
    let rust = native_unbound_fixture(&bytes, label);
    let root_model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native root ViewModel");
    let owned = ViewModel::create_instance_handle(&root_model).expect("owned ViewModelInstance");
    let child = owned
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("root ViewModelInstance")
                .property_value_named("child")
        })
        .flatten()
        .and_then(|property| {
            property
                .with(|property| {
                    property
                        .as_view_model_instance_view_model()
                        .expect("ViewModelInstanceViewModel")
                        .reference_view_model_instance()
                })
                .flatten()
        })
        .expect("generated child ViewModelInstance");
    let boolean = child
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("child ViewModelInstance")
                .property_value_named("enabled")
        })
        .flatten()
        .expect("nested boolean property");
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    assert!(CoreRegistry::set_bool_handle(
        &boolean,
        i32::from(ViewModelInstanceBooleanBase::PROPERTY_VALUE_PROPERTY_KEY),
        true,
    ));
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

fn assert_default_scalar_to_number_converter(probe: &std::path::Path, label: &str, bytes: Vec<u8>) {
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    bind_default_root_view_model(&rust, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_number_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_bind_cpp.riv";
    let bytes = fixture_bytes(8370);
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == 1)
        })
        .and_then(|component| component.local_transform.map(|transform| transform[4]))
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn state_machine_external_viewmodel_number_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_external_viewmodel_number_bind_cpp.riv";
    let bytes = fixture_bytes(8387);
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let external = rust
        ._file
        .with_file(|file| file.view_model(0))
        .and_then(|model| {
            model
                .with(|model| model.as_view_model().expect("ViewModel").instance_at(1))
                .flatten()
        })
        .expect("serialized external view-model instance");
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(external));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == 1)
        })
        .and_then(|component| component.local_transform.map(|transform| transform[4]))
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn state_machine_owned_viewmodel_number_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_bind_cpp.riv";
    let bytes = fixture_bytes(8395);
    let value = 0.25_f32;
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native ViewModel 0");
    let owned = ViewModel::create_instance_handle(&model).expect("owned ViewModelInstance");
    let number = owned
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("ViewModelInstance")
                .property_value_named("amount")
        })
        .flatten()
        .expect("owned number property");
    number
        .with_mut(|number| {
            number
                .as_view_model_instance_number_mut()
                .expect("ViewModelInstanceNumber")
                .set_value(value)
        })
        .expect("live owned number property");
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == 1)
        })
        .and_then(|component| component.local_transform.map(|transform| transform[4]))
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn state_machine_owned_viewmodel_number_source_handle_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_source_handle_bind_cpp.riv";
    let bytes = fixture_bytes(8762);
    let value = 0.25_f32;
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native ViewModel 0");
    let owned = ViewModel::create_instance_handle(&model).expect("owned ViewModelInstance");
    let number = owned
        .with(|instance| {
            let instance = instance
                .as_view_model_instance()
                .expect("ViewModelInstance");
            assert!(instance.property_value_named("child/amount").is_none());
            instance.property_value_named("amount")
        })
        .flatten()
        .expect("owned number source handle");
    let key = i32::from(ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
    assert!(CoreRegistry::set_double_handle(&number, key, value));
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == 1)
        })
        .and_then(|component| component.local_transform.map(|transform| transform[4]))
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn state_machine_imported_viewmodel_number_public_update_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_imported_number_public_update_target_to_source_cpp.riv";
    let bytes = number_two_way_fixture_bytes(10033);
    let target_value = 4.46_f32;
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        target_value.to_string(),
        "--runtime-update-state-machine-data-binds".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("native ViewModel 0");
    let imported = model
        .with(|model| model.as_view_model().expect("ViewModel").instance_at(0))
        .flatten()
        .expect("imported ViewModelInstance 0");
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(imported.clone()));
    let definition = rust
        .machine
        .with_instance(|machine| machine.state_machine());
    let authored_bind = definition
        .with_downcast::<StateMachine, _>(|machine| machine.data_bind(0))
        .flatten()
        .expect("authored DataBind 0");
    let authored_target = authored_bind
        .with(|bind| bind.as_data_bind().and_then(DataBind::target))
        .flatten()
        .expect("authored BindablePropertyNumber");
    let target = rust
        .machine
        .with_instance(|machine| machine.bindable_property_instance(&authored_target))
        .expect("runtime BindablePropertyNumber");
    let to_source = rust
        .machine
        .with_instance(|machine| machine.bindable_data_bind_to_target(&target))
        .expect("runtime target-to-source DataBind");
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    assert!(CoreRegistry::set_double_handle(
        &target,
        i32::from(BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
        target_value,
    ));
    DataBind::update_data_bind_handle(&to_source, true);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[2],
        &rust.machine,
        advanced,
        label,
    );
    let source_value = imported
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("ViewModelInstance")
                .property_values()
                .first()
                .cloned()
        })
        .flatten()
        .and_then(|number| {
            number.with_downcast::<
                nuxie_runtime::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
                _,
            >(|number| number.value())
        })
        .expect("imported source number");
    assert_close(source_value, target_value, label);
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
}

#[test]
fn state_machine_default_viewmodel_boolean_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_boolean_bind_cpp.riv";
    let bytes = boolean_fixture_bytes(8371);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_fixture_with_default_binding(&bytes, label, false);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (index, (cpp_advance, seconds)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
        .enumerate()
    {
        if index == 1 {
            rust.machine.with_instance_mut(|machine| {
                machine.bind_view_model_instance(
                    rust.default_view_model
                        .as_ref()
                        .expect("default view model")
                        .clone(),
                )
            });
        }
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id == 1)
        })
        .and_then(|component| component.local_transform.map(|transform| transform[4]))
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn boolean_public_update_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_boolean_public_update_cpp.riv";
    let bytes = boolean_fixture_bytes_with_flags(8644, 1 << 1);
    let forced_value = false;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        forced_value.to_string(),
        "--runtime-update-state-machine-data-binds".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_fixture(&bytes, label);
    let definition = rust
        .machine
        .with_instance(|machine| machine.state_machine());
    let authored_bind = definition
        .with_downcast::<StateMachine, _>(|machine| machine.data_bind(0))
        .flatten()
        .expect("authored DataBind 0");
    let authored_target = authored_bind
        .with(|bind| bind.as_data_bind().and_then(DataBind::target))
        .flatten()
        .expect("authored BindablePropertyBoolean");
    let target = rust
        .machine
        .with_instance(|machine| machine.bindable_property_instance(&authored_target))
        .expect("runtime BindablePropertyBoolean");
    let to_source = rust
        .machine
        .with_instance(|machine| machine.bindable_data_bind_to_target(&target))
        .expect("runtime two-way DataBind");
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    assert!(CoreRegistry::set_bool_handle(
        &target,
        i32::from(BindablePropertyBooleanBase::PROPERTY_VALUE_PROPERTY_KEY),
        forced_value,
    ));
    DataBind::update_data_bind_handle(&to_source, true);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    let source_value = rust
        .default_view_model
        .as_ref()
        .expect("default ViewModelInstance")
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("ViewModelInstance")
                .property_values()
                .first()
                .cloned()
        })
        .flatten()
        .and_then(|value| {
            value.with(|value| {
                value
                    .as_view_model_instance_boolean()
                    .map(|value| value.value())
            })
        })
        .flatten()
        .expect("default boolean source");
    assert_eq!(source_value, forced_value);
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
}

#[test]
fn state_machine_owned_viewmodel_number_name_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_name_bind_cpp.riv";
    let value = 0.25_f32;
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    assert_owned_number_bind(&probe, label, fixture_bytes(8577), &args, false);
}

#[test]
fn state_machine_owned_viewmodel_nested_number_name_path_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_number_name_path_bind_cpp.riv";
    let value = 0.25_f32;
    let args = [
        "--runtime-bind-owned-view-model-number-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/amount".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    assert_owned_number_bind(
        &probe,
        label,
        nested_number_fixture_bytes(8578),
        &args,
        true,
    );
}

#[test]
fn state_machine_owned_viewmodel_nested_number_source_handle_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_number_source_handle_bind_cpp.riv";
    let value = 0.25_f32;
    let args = [
        "--runtime-bind-owned-view-model-number-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/amount".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    assert_owned_number_bind(
        &probe,
        label,
        nested_number_fixture_bytes(8784),
        &args,
        true,
    );
}

#[test]
fn state_machine_owned_viewmodel_imported_intermediate_number_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_imported_intermediate_number_source_cpp.riv";
    let bytes = imported_intermediate_number_fixture_bytes(8588);
    let args = [
        "--runtime-bind-owned-view-model-viewmodel-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let owned = owned_context_with_imported_child(&rust);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    rust.machine
        .with_instance_mut(|machine| machine.advanced_data_context());
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        false,
        label,
    );
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        advanced,
        label,
    );
}

#[test]
fn state_machine_owned_viewmodel_imported_intermediate_boolean_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_imported_intermediate_boolean_source_cpp.riv";
    let bytes = imported_intermediate_boolean_fixture_bytes(8589);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-viewmodel-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    let owned = owned_context_with_imported_child(&rust);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .filter_map(|component| component.local_transform.map(|transform| transform[4]))
                .last()
        })
        .expect("C++ bound transform x");
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| {
            node.with(|node| node.as_node().map(|node| node.x()))
                .flatten()
        })
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

#[test]
fn state_machine_default_viewmodel_nested_number_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_nested_number_bind_cpp.riv";
    let bytes = nested_number_fixture_bytes_with_value(8582, 0.25);
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    bind_default_root_view_model(&rust, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_nested_boolean_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_nested_boolean_bind_cpp.riv";
    let bytes = nested_boolean_fixture_bytes(8583, true);
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    bind_default_root_view_model(&rust, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_owned_viewmodel_nested_boolean_name_path_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_boolean_name_path_bind_cpp.riv";
    let bytes = nested_boolean_fixture_bytes(8579, false);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-bool-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/enabled".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    assert_owned_nested_boolean_bind(&probe, label, bytes, &args);
}

#[test]
fn state_machine_owned_viewmodel_nested_boolean_source_handle_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_boolean_source_handle_bind_cpp.riv";
    let bytes = nested_boolean_fixture_bytes(8785, false);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-bool-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/enabled".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    assert_owned_nested_boolean_bind(&probe, label, bytes, &args);
}

#[test]
fn state_machine_default_viewmodel_boolean_to_number_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_boolean_to_number_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_boolean_to_number_converter_blend_state(8407),
    );
}

#[test]
fn state_machine_default_viewmodel_enum_to_number_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_enum_to_number_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_enum_to_number_converter_blend_state(8408),
    );
}

#[test]
fn state_machine_default_viewmodel_color_to_number_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_color_to_number_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_color_to_number_converter_blend_state(8409),
    );
}

#[test]
fn state_machine_default_viewmodel_string_to_number_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_string_to_number_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_string_to_number_converter_blend_state(8410),
    );
}

#[test]
fn state_machine_default_viewmodel_symbol_list_index_to_number_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_symbol_list_index_to_number_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_symbol_list_index_to_number_converter_blend_state(
            8423,
        ),
    );
}

#[test]
fn state_machine_default_viewmodel_symbol_list_index_operation_value_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    assert_default_scalar_to_number_converter(
        &probe,
        "synthetic/runtime_state_machine_default_viewmodel_symbol_list_index_operation_value_converter_cpp.riv",
        synthetic_state_machine_default_viewmodel_symbol_list_index_operation_value_blend_state(
            8424,
        ),
    );
}

#[test]
fn state_machine_default_viewmodel_number_operation_value_converter_cases_0_through_3_match_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let cases = [(0, 0.4, 0.2), (1, 0.8, 0.3), (2, 0.4, 2.0), (3, 0.8, 2.0)];
    for (case_index, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_operation_value_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
                8433 + case_index as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

#[test]
fn state_machine_default_viewmodel_number_operation_value_converter_cases_4_through_7_match_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let cases = [(4, 1.3, 1.0), (5, 0.25, 1.0), (6, 0.5, 2.0), (7, 0.0, 1.0)];
    for (case_offset, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_operation_value_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
                8437 + case_offset as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

#[test]
fn state_machine_default_viewmodel_number_operation_value_converter_cases_8_through_11_match_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let cases = [(8, 1.0, 1.0), (9, 0.0, 1.0), (10, 0.0, 1.0), (11, 0.0, 1.0)];
    for (case_offset, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_operation_value_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
                8441 + case_offset as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

#[test]
fn state_machine_default_viewmodel_number_operation_value_converter_cases_12_through_15_match_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let cases = [
        (12, 1.0, 1.0),
        (13, 0.0, 1.0),
        (14, 0.0, 1.0),
        (15, 1.0, 1.0),
    ];
    for (case_offset, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_operation_value_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
                8445 + case_offset as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

#[test]
fn state_machine_default_viewmodel_number_operation_value_converter_cases_16_through_18_match_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let cases = [(16, 0.6, 1.0), (17, 0.6, 1.0), (18, 0.4, 1.0)];
    for (case_offset, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_operation_value_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_operation_value_blend_state(
                8449 + case_offset as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

#[test]
fn state_machine_default_viewmodel_number_formula_converter_matrix_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let cases = [
        (0, 0.4, 0.2),
        (1, 0.8, 0.3),
        (2, 0.4, 2.0),
        (3, 0.8, 2.0),
        (4, 1.3, 1.0),
        (99, 0.8, 2.0),
    ];
    for (case_index, (operation_type, source_value, operation_value)) in
        cases.into_iter().enumerate()
    {
        let label = format!(
            "synthetic/runtime_state_machine_default_viewmodel_number_formula_converter_{operation_type}_cpp.riv"
        );
        assert_default_scalar_to_number_converter(
            &probe,
            &label,
            synthetic_state_machine_default_viewmodel_number_formula_blend_state(
                8463 + case_index as u64,
                source_value,
                operation_type,
                operation_value,
            ),
        );
    }
}

fn formula_number_property(instance: &CoreHandle) -> CoreHandle {
    instance
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("ViewModelInstance")
                .property_value_named("amount")
        })
        .flatten()
        .expect("formula number source")
}

fn set_formula_number(property: &CoreHandle, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        property,
        i32::from(ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
        value,
    ));
}

fn compare_formula_steps(cpp: &CppProbeFile, rust: &NativeFixture, seconds: &[f32], label: &str) {
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds.iter().copied())
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, rust, label);
}

fn formula_runtime_target(rust: &NativeFixture) -> (CoreHandle, Option<CoreHandle>) {
    let definition = rust
        .machine
        .with_instance(|machine| machine.state_machine());
    let authored_bind = definition
        .with_downcast::<StateMachine, _>(|machine| machine.data_bind(0))
        .flatten()
        .expect("formula DataBind 0");
    let authored_target = authored_bind
        .with(|bind| bind.as_data_bind().and_then(DataBind::target))
        .flatten()
        .expect("formula authored target");
    let target = rust
        .machine
        .with_instance(|machine| machine.bindable_property_instance(&authored_target))
        .expect("formula runtime target");
    let bind = rust.machine.with_instance(|machine| {
        machine
            .bindable_data_bind_to_source(&target)
            .or_else(|| machine.bindable_data_bind_to_target(&target))
    });
    (target, bind)
}

fn assert_formula_reverse_flow(
    probe: &std::path::Path,
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    initial_data_context: bool,
    public_update: bool,
    target_value: f32,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, &bytes, args);
    let rust = native_unbound_fixture(&bytes, label);
    bind_default_root_view_model(&rust, label);
    let (target, bind) = formula_runtime_target(&rust);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);
    if initial_data_context {
        rust.machine
            .with_instance_mut(|machine| machine.advanced_data_context());
        compare_advance(
            &cpp_artboard.runtime_state_machine_advances[0],
            &rust.machine,
            false,
            label,
        );
    } else {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(0.0));
        compare_advance(
            &cpp_artboard.runtime_state_machine_advances[0],
            &rust.machine,
            advanced,
            label,
        );
    }
    assert!(CoreRegistry::set_double_handle(
        &target,
        i32::from(BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
        target_value
    ));
    if public_update {
        DataBind::update_data_bind_handle(bind.as_ref().expect("formula runtime DataBind"), true);
    } else {
        DataBind::update_data_bind_handle(bind.as_ref().expect("formula runtime DataBind"), true);
        rust.machine
            .with_instance_mut(|machine| machine.advanced_data_context());
    }
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_imported_viewmodel_number_formula_context_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_number_formula_context_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        8732, 1.0, 2, 0.5, 0, true,
    );
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-view-model-instance-source-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.25".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let imported = rust
        ._file
        .with_file(|file| file.create_view_model_instance_at(0, 0))
        .expect("imported instance");
    set_formula_number(&formula_number_property(&imported), 0.25);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(imported));
    compare_formula_steps(&cpp, &rust, &[0.0, 1.0], label);
}

#[test]
fn state_machine_owned_viewmodel_number_formula_context_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_formula_context_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        8733, 1.0, 2, 0.5, 0, true,
    );
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.25".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("ViewModel");
    let owned = ViewModel::create_instance_handle(&model).expect("owned instance");
    set_formula_number(&formula_number_property(&owned), 0.25);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    compare_formula_steps(&cpp, &rust, &[0.0, 1.0], label);
}

#[test]
fn state_machine_owned_viewmodel_number_formula_source_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_number_formula_source_mutation_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        9307, 1.0, 2, 0.5, 0, true,
    );
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.25".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-owned-view-model-source-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.75".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_unbound_fixture(&bytes, label);
    let model = rust
        ._file
        .with_file(|file| file.view_model(0))
        .expect("ViewModel");
    let owned = ViewModel::create_instance_handle(&model).expect("owned instance");
    let number = formula_number_property(&owned);
    set_formula_number(&number, 0.25);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    set_formula_number(&number, 0.75);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
    }
    compare_runtime_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_number_formula_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_target_to_source_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        8494, 1.5, 2, 2.0, 3, true,
    );
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "0.4",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(&probe, label, bytes, &args, true, false, 0.4);
}

#[test]
fn state_machine_default_viewmodel_number_formula_main_to_target_two_way_target_to_source_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_main_to_target_two_way_target_to_source_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        8495, 1.5, 2, 2.0, 2, false,
    );
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "0.4",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(&probe, label, bytes, &args, false, false, 0.4);
}

#[test]
fn state_machine_default_viewmodel_number_formula_public_update_target_to_source_matches_cpp_probe()
{
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_public_update_target_to_source_cpp.riv";
    let bytes = synthetic_state_machine_default_viewmodel_number_formula_blend_state_with_flags(
        8531, 1.5, 2, 2.0, 2, false,
    );
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "0.4",
        "--runtime-update-state-machine-data-binds",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(&probe, label, bytes, &args, false, true, 0.4);
}

#[test]
fn state_machine_default_viewmodel_number_formula_group_public_update_target_to_source_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_group_public_update_target_to_source_cpp.riv";
    let bytes =
        synthetic_state_machine_default_viewmodel_number_formula_group_blend_state_with_flags(
            8638, 2,
        );
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "4.46",
        "--runtime-update-state-machine-data-binds",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(&probe, label, bytes, &args, false, true, 4.46);
}

fn formula_function_fixture(file_id: u64, flags: u64, observer: bool) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags(
        file_id,
        1.5,
        6,
        &[
            FormulaFunctionArgument::Input,
            FormulaFunctionArgument::Value(2.0),
        ],
        flags,
        observer,
    )
}

#[test]
fn state_machine_default_viewmodel_number_formula_function_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_function_target_to_source_cpp.riv";
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "0.4",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(
        &probe,
        label,
        formula_function_fixture(8658, 3, true),
        &args,
        true,
        false,
        0.4,
    );
}

#[test]
fn state_machine_default_viewmodel_number_formula_function_public_update_target_to_source_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_function_public_update_target_to_source_cpp.riv";
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "0.4",
        "--runtime-update-state-machine-data-binds",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(
        &probe,
        label,
        formula_function_fixture(8659, 2, false),
        &args,
        false,
        true,
        0.4,
    );
}

#[test]
fn state_machine_default_viewmodel_number_formula_function_main_to_target_two_way_target_dirty_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else { return };
    let label = "synthetic/runtime_state_machine_default_viewmodel_number_formula_function_main_to_target_two_way_target_dirty_cpp.riv";
    let args = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-state-machine-bindable-number",
        "0",
        "0",
        "4.46",
        "--runtime-advance-state-machine-data-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ]
    .map(str::to_owned);
    assert_formula_reverse_flow(
        &probe,
        label,
        formula_function_fixture(8660, 2, false),
        &args,
        false,
        false,
        4.46,
    );
}

fn counted_random_args(values: &[f32], actions: &[&str]) -> Vec<String> {
    let mut args = vec!["--runtime-random-reset".to_owned()];
    for value in values {
        args.extend(["--runtime-random-value".to_owned(), value.to_string()]);
    }
    args.extend(actions.iter().map(|value| (*value).to_owned()));
    args
}

fn assert_random_reverse(label: &str, bytes: Vec<u8>, values: &[f32], public: bool) {
    let _guard = RANDOM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let actions = if public {
        vec![
            "--runtime-bind-default-view-model-state-machine-context",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "0",
            "--runtime-set-state-machine-bindable-number",
            "0",
            "0",
            "0.4",
            "--runtime-update-state-machine-data-binds",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "1",
        ]
    } else {
        vec![
            "--runtime-bind-default-view-model-state-machine-context",
            "0",
            "--runtime-set-state-machine-bindable-number",
            "0",
            "0",
            "0.4",
            "--runtime-advance-state-machine-data-context",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "1",
        ]
    };
    let cpp = read_cpp_probe_bytes_with_args(
        &probe_path().expect("C++ probe"),
        label,
        &bytes,
        &counted_random_args(values, &actions),
    );
    let board = cpp.artboards.first().expect("C++ artboard");
    let rust = native_unbound_fixture(&bytes, label);
    bind_default_root_view_model(&rust, label);
    RandomProvider::clear_randoms();
    for value in values {
        RandomProvider::add_random_value(*value);
    }
    let (target, bind) = formula_runtime_target(&rust);
    let mut report = 0;
    if public {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(0.0));
        compare_advance(
            &board.runtime_state_machine_advances[report],
            &rust.machine,
            advanced,
            label,
        );
        report += 1;
    }
    assert!(CoreRegistry::set_double_handle(
        &target,
        i32::from(BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
        0.4
    ));
    DataBind::update_data_bind_handle(bind.as_ref().expect("random DataBind"), true);
    if !public {
        rust.machine
            .with_instance_mut(|machine| machine.advanced_data_context());
    }
    compare_advance(
        &board.runtime_state_machine_advances[report],
        &rust.machine,
        false,
        label,
    );
    report += 1;
    for seconds in [0.0, 1.0] {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(
            &board.runtime_state_machine_advances[report],
            &rust.machine,
            advanced,
            label,
        );
        report += 1;
    }
    assert_eq!(
        board
            .runtime_state_machine_advances
            .last()
            .expect("report")
            .random_total_calls,
        RandomProvider::total_calls()
    );
    compare_runtime_node_x(board, &rust, label);
    RandomProvider::clear_testing_mode();
}

fn random_reverse_fixture(file_id: u64, mode: u64, flags: u64, observer: bool) -> Vec<u8> {
    synthetic_state_machine_default_viewmodel_number_formula_function_blend_state_with_flags_and_random_mode(file_id, 0.0, 16, &[FormulaFunctionArgument::Value(2.0), FormulaFunctionArgument::Value(6.0)], flags, observer, mode)
}

fn random_group_fixture(file_id: u64, mode: u64, flags: u64, observer: bool) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.75);
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_f32_property(bytes, "DataConverterOperationValue", "operationValue", 2.0);
        });
        push_object_with_properties(bytes, "DataConverterFormula", |bytes| {
            if mode != 0 {
                push_uint_property(bytes, "DataConverterFormula", "randomModeValue", mode);
            }
        });
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", 16);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_f32_property(bytes, "FormulaTokenValue", "operationValue", 2.0);
        });
        push_object_with_properties(bytes, "FormulaTokenArgumentSeparator", |_| {});
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_f32_property(bytes, "FormulaTokenValue", "operationValue", 6.0);
        });
        push_object_with_properties(bytes, "FormulaTokenParenthesisClose", |_| {});
        push_object_with_properties(bytes, "DataConverterGroup", |_| {});
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_bindable_number_data_bind_context_with_converter_and_flags(
            bytes,
            0.0,
            &[0, 0],
            Some(2),
            flags,
        );
        if observer {
            push_bindable_number_data_bind_context(bytes, 0.0, &[0, 0]);
        }
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn assert_random_group_forward(label: &str, bytes: Vec<u8>, values: &[f32]) {
    let _guard = RANDOM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let actions = [
        "--runtime-bind-default-view-model-state-machine-context",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-set-default-view-model-source-number",
        "0",
        "0",
        "1",
        "--runtime-advance-state-machine",
        "0",
        "0",
        "--runtime-advance-state-machine",
        "0",
        "1",
    ];
    let cpp = read_cpp_probe_bytes_with_args(
        &probe_path().expect("C++ probe"),
        label,
        &bytes,
        &counted_random_args(values, &actions),
    );
    let board = cpp.artboards.first().expect("C++ artboard");
    let rust = native_unbound_fixture(&bytes, label);
    let instance = bind_default_root_view_model(&rust, label);
    RandomProvider::clear_randoms();
    for value in values {
        RandomProvider::add_random_value(*value);
    }
    let first = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &board.runtime_state_machine_advances[0],
        &rust.machine,
        first,
        label,
    );
    set_formula_number(&formula_number_property(&instance), 1.0);
    for (report, seconds) in board.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(report, &rust.machine, advanced, label);
    }
    assert_eq!(
        board
            .runtime_state_machine_advances
            .last()
            .unwrap()
            .random_total_calls,
        RandomProvider::total_calls() as i32,
        "{label} random call count"
    );
    compare_runtime_node_x(board, &rust, label);
    RandomProvider::clear_testing_mode();
}

#[test]
fn random_function_group_native() {
    assert_random_group_forward(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_cpp.riv",
        random_group_fixture(8800, 0, 0, false),
        &[0.25],
    );
}

#[test]
fn random_function_group_always_native() {
    assert_random_group_forward(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_always_cpp.riv",
        random_group_fixture(8806, 1, 0, false),
        &[0.25, 0.75],
    );
}

#[test]
fn random_function_group_source_change_native() {
    assert_random_group_forward(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_source_change_cpp.riv",
        random_group_fixture(8807, 2, 0, false),
        &[0.25, 0.75],
    );
}

#[test]
fn random_function_group_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_target_to_source_cpp.riv",
        random_group_fixture(8803, 0, 3, true),
        &[0.25],
        false,
    );
}

#[test]
fn random_function_group_always_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_always_target_to_source_cpp.riv",
        random_group_fixture(8817, 1, 3, true),
        &[0.25, 0.75],
        false,
    );
}

#[test]
fn random_function_group_source_change_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_group_source_change_target_to_source_cpp.riv",
        random_group_fixture(8820, 2, 3, true),
        &[0.25, 0.75],
        false,
    );
}

#[test]
fn random_function_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_target_to_source_cpp.riv",
        random_reverse_fixture(8664, 0, 3, true),
        &[0.25],
        false,
    );
}
#[test]
fn random_function_always_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_always_target_to_source_cpp.riv",
        random_reverse_fixture(8811, 1, 3, true),
        &[0.25, 0.75, 0.5],
        false,
    );
}
#[test]
fn random_function_source_change_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_source_change_target_to_source_cpp.riv",
        random_reverse_fixture(8814, 2, 3, true),
        &[0.25, 0.75, 0.5],
        false,
    );
}
#[test]
fn random_function_public_update_target_to_source_native() {
    assert_random_reverse(
        "synthetic/runtime_state_machine_default_viewmodel_number_formula_random_function_public_update_target_to_source_cpp.riv",
        random_reverse_fixture(8665, 0, 2, false),
        &[0.25],
        true,
    );
}
