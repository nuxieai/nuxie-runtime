//! Enum data-bind differential observed directly from retained native owners.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine::StateMachine,
    artboard::Artboard as NativeArtboard,
    data_bind::{
        bindable_property_enum::BindablePropertyEnum,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        converters::{
            data_converter_formula::DataConverterFormula, data_converter_group::DataConverterGroup,
            data_converter_operation_viewmodel::DataConverterOperationViewModel,
            data_converter_to_number::DataConverterToNumber,
            data_converter_to_string::DataConverterToString,
        },
        data_bind_container::DataBindContainerOwner,
    },
    generated::{
        core_registry::CoreRegistry,
        data_bind::bindable_property_enum_base::BindablePropertyEnumBase,
        data_bind::bindable_property_number_base::BindablePropertyNumberBase,
        data_bind::bindable_property_string_base::BindablePropertyStringBase,
        viewmodel::viewmodel_instance_enum_base::ViewModelInstanceEnumBase,
        viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    },
    node::Node,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
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
    #[serde(rename = "enumBindings", default)]
    enum_bindings: Vec<CppRuntimeStateMachineEnumBinding>,
    #[serde(rename = "numberBindings", default)]
    number_bindings: Vec<CppRuntimeStateMachineNumberBinding>,
    #[serde(rename = "stringBindings", default)]
    string_bindings: Vec<CppRuntimeStateMachineStringBinding>,
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
struct CppRuntimeStateMachineEnumBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "sourceValue")]
    source_value: Option<u64>,
    #[serde(rename = "targetValue")]
    target_value: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineNumberBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "sourceValue")]
    source_value: Option<f32>,
    #[serde(rename = "targetValue")]
    target_value: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineStringBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "sourceValue")]
    source_value: Option<String>,
    #[serde(rename = "targetValue")]
    target_value: Option<String>,
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
    default_view_model: CoreHandle,
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

fn push_bindable_enum_data_bind_context(bytes: &mut Vec<u8>, value: u64, path: &[u32]) {
    push_bindable_enum_data_bind_context_with_flags(bytes, value, path, 0);
}

fn push_bindable_enum_data_bind_context_with_flags(
    bytes: &mut Vec<u8>,
    value: u64,
    path: &[u32],
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyEnum", |bytes| {
        push_uint_property(bytes, "BindablePropertyEnum", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyEnum",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn target_to_source_fixture_bytes(file_id: u64, initial_value: u64, flags: &[u64]) -> Vec<u8> {
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
            push_uint_property(
                bytes,
                "ViewModelInstanceEnum",
                "propertyValue",
                initial_value,
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
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        for flag in flags {
            push_bindable_enum_data_bind_context_with_flags(bytes, 0, &[0, 0], *flag);
        }
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueEnumComparator", |bytes| {
            push_uint_property(bytes, "TransitionValueEnumComparator", "value", 1);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_bindable_number_data_bind_context_with_converter(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    converter_id: u64,
) {
    push_bindable_number_data_bind_context_with_converter_and_flags(
        bytes,
        value,
        path,
        converter_id,
        0,
    );
}

fn push_bindable_number_data_bind_context_with_converter_and_flags(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    converter_id: u64,
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
        push_uint_property(bytes, "DataBindContext", "converterId", converter_id);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn enum_to_number_fixture_bytes(file_id: u64, flags: u64) -> Vec<u8> {
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
            0,
            flags,
        );
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn enum_operation_fixture_bytes(file_id: u64, grouped: bool) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
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
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.4);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 1);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 0);
        });
        let operation_converter_id = if grouped { 1 } else { 0 };
        if grouped {
            push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
                push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
                push_f32_property(bytes, "DataConverterOperationValue", "operationValue", 2.0);
            });
        }
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            let mut source_path_ids = Vec::new();
            push_var_uint(&mut source_path_ids, 0);
            push_var_uint(&mut source_path_ids, 1);
            push_uint_property(bytes, "DataConverterOperationViewModel", "operationType", 0);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &source_path_ids,
            );
        });
        if grouped {
            push_object_with_properties(bytes, "DataConverterGroup", |_| {});
            push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
                push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
            });
            push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
                push_uint_property(
                    bytes,
                    "DataConverterGroupItem",
                    "converterId",
                    operation_converter_id,
                );
            });
        }
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
        push_bindable_number_data_bind_context_with_converter(
            bytes,
            0.0,
            &[0, 0],
            if grouped { 2 } else { 0 },
        );
        push_bindable_enum_data_bind_context(bytes, 0, &[0, 1]);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 2.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_bindable_string_data_bind_context_with_converter_and_flags(
    bytes: &mut Vec<u8>,
    value: &str,
    path: &[u32],
    converter_id: u64,
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyString", |bytes| {
        push_string_property(bytes, "BindablePropertyString", "propertyValue", value);
    });
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
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        push_uint_property(bytes, "DataBindContext", "converterId", converter_id);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn enum_to_string_fixture_bytes(file_id: u64, flags: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_string_property(bytes, "DataEnumCustom", "name", "Choice");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "");
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
        push_object_with_properties(bytes, "DataConverterToString", |_| {});
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
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_bindable_string_data_bind_context_with_converter_and_flags(
            bytes,
            "",
            &[0, 0],
            0,
            flags,
        );
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueStringComparator", |bytes| {
            push_string_property(
                bytes,
                "TransitionValueStringComparator",
                "value",
                "Second Label",
            );
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn formula_enum_fixture_bytes(file_id: u64) -> Vec<u8> {
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
        push_object_with_properties(bytes, "DataConverterFormula", |_| {});
        push_object_with_properties(bytes, "FormulaTokenInput", |_| {});
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
        push_bindable_number_data_bind_context_with_converter(bytes, 0.75, &[0, 0], 0);
        push_bindable_enum_data_bind_context(bytes, 0, &[0, 0]);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn fixture_bytes(file_id: u64) -> Vec<u8> {
    fixture_bytes_with_state_machines(file_id, 1)
}

fn fixture_bytes_with_state_machines(file_id: u64, state_machine_count: usize) -> Vec<u8> {
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
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "alternate");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 0);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 0);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        for _ in 0..state_machine_count {
            push_enum_state_machine(bytes, &[0, 0]);
        }
    })
}

fn push_enum_state_machine(bytes: &mut Vec<u8>, source_path: &[u32]) {
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
    push_bindable_enum_data_bind_context(bytes, 0, source_path);
    push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
        push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
    });
    push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
    push_object_with_properties(bytes, "TransitionValueEnumComparator", |bytes| {
        push_uint_property(bytes, "TransitionValueEnumComparator", "value", 1);
    });
    push_object_with_properties(bytes, "AnimationState", |bytes| {
        push_uint_property(bytes, "AnimationState", "animationId", 1);
    });
    push_object_with_properties(bytes, "ExitState", |_| {});
}

fn nested_fixture_bytes(
    file_id: u64,
    imported_child_value: u64,
    state_machine_count: usize,
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
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "choice");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 0);
            push_uint_property(
                bytes,
                "ViewModelInstanceEnum",
                "propertyValue",
                imported_child_value,
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
        for _ in 0..state_machine_count {
            push_enum_state_machine(bytes, &[0, 0, 0]);
        }
    })
}

fn native_fixture(bytes: &[u8], label: &str) -> NativeFixture {
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
    let default_view_model = file
        .with_file(|file| file.view_model_handle(0))
        .and_then(|view_model| {
            view_model.with_downcast::<ViewModel, _>(ViewModel::default_instance)
        })
        .flatten()
        .unwrap_or_else(|| panic!("missing native default view model for {label}"));
    NativeFixture {
        _file: file,
        artboard,
        machine,
        default_view_model,
    }
}

fn native_enum_binding_at(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
) -> (Option<CoreHandle>, Option<u64>) {
    machine.with_instance(|machine| {
        let target = native_enum_target_for_instance_at(machine, data_bind_index);
        let bind = machine
            .bindable_data_bind_to_target(&target)
            .or_else(|| machine.bindable_data_bind_to_source(&target))
            .expect("native enum bind occurrence");
        let source = bind
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.source()))
            .flatten();
        let target = target.with_downcast::<BindablePropertyEnum, _>(|target| {
            u64::from(target.base.property_value())
        });
        (source, target)
    })
}

fn native_number_binding_at(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
) -> (Option<CoreHandle>, Option<f32>, Option<CoreHandle>) {
    machine.with_instance(|machine| {
        let target = native_enum_target_for_instance_at(machine, data_bind_index);
        let bind = machine
            .bindable_data_bind_to_target(&target)
            .or_else(|| machine.bindable_data_bind_to_source(&target))
            .expect("native number bind occurrence");
        let (source, converter) = bind
            .with(|bind| {
                let bind = bind.as_data_bind()?;
                Some((bind.source(), bind.converter()))
            })
            .flatten()
            .expect("native number DataBind");
        let target = target
            .with_downcast::<BindablePropertyNumber, _>(|target| target.base.property_value());
        (source, target, converter)
    })
}

fn native_string_binding_at(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
) -> (Option<CoreHandle>, Option<String>, Option<CoreHandle>) {
    machine.with_instance(|machine| {
        let target = native_enum_target_for_instance_at(machine, data_bind_index);
        let bind = machine
            .bindable_data_bind_to_target(&target)
            .or_else(|| machine.bindable_data_bind_to_source(&target))
            .expect("native string bind occurrence");
        let (source, converter) = bind
            .with(|bind| {
                let bind = bind.as_data_bind()?;
                Some((bind.source(), bind.converter()))
            })
            .flatten()
            .expect("native string DataBind");
        let target = target.with_downcast::<BindablePropertyString, _>(|target| {
            target.base.property_value().to_owned()
        });
        (source, target, converter)
    })
}

fn set_native_number_target(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
    value: f32,
) {
    let target = machine
        .with_instance(|machine| native_enum_target_for_instance_at(machine, data_bind_index));
    assert!(CoreRegistry::set_double_handle(
        &target,
        BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value,
    ));
    assert_eq!(
        target.with_downcast::<BindablePropertyNumber, _>(|target| target.base.property_value()),
        Some(value),
        "native number target write"
    );
}

fn set_native_string_target(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
    value: &str,
) {
    let target = machine
        .with_instance(|machine| native_enum_target_for_instance_at(machine, data_bind_index));
    assert!(CoreRegistry::set_string_handle(
        &target,
        BindablePropertyStringBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value.to_owned(),
    ));
    assert_eq!(
        target.with_downcast::<BindablePropertyString, _>(|target| {
            target.base.property_value().to_owned()
        }),
        Some(value.to_owned()),
        "native string target write"
    );
}

fn native_enum_target_for_instance_at(
    machine: &nuxie_runtime::source::animation::state_machine_instance::StateMachineInstance,
    data_bind_index: usize,
) -> CoreHandle {
    let authored_bind = machine
        .state_machine()
        .with_downcast::<StateMachine, _>(|state_machine| state_machine.data_bind(data_bind_index))
        .flatten()
        .expect("authored enum data bind");
    let authored_target = authored_bind
        .with(|bind| bind.as_data_bind().and_then(|bind| bind.target()))
        .flatten()
        .expect("authored enum bind target");
    machine
        .bindable_property_instance(&authored_target)
        .expect("native enum bind target occurrence")
}

fn set_native_enum_target(machine: &RuntimeStateMachineInstanceHandle, value: u64) {
    set_native_enum_target_at(machine, 0, value);
}

fn set_native_enum_target_at(
    machine: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
    value: u64,
) {
    let target = machine
        .with_instance(|machine| native_enum_target_for_instance_at(machine, data_bind_index));
    CoreRegistry::set_uint_handle(
        &target,
        BindablePropertyEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value as u32,
    );
    assert_eq!(
        target.with_downcast::<BindablePropertyEnum, _>(|target| target.base.property_value()),
        Some(value as u32),
        "native enum target write"
    );
}

fn default_enum_value(view_model: &CoreHandle) -> u64 {
    enum_source_value(&native_enum_property(view_model))
}

fn enum_source_value(property: &CoreHandle) -> u64 {
    property
        .with_downcast::<ViewModelInstanceEnum, _>(|value| u64::from(value.base.property_value()))
        .expect("native enum value")
}

fn native_enum_property(view_model: &CoreHandle) -> CoreHandle {
    let value = view_model
        .with_downcast::<ViewModelInstance, _>(|view_model| view_model.property_value_by_id(0))
        .flatten()
        .expect("native enum property");
    assert_eq!(
        value.with_downcast::<ViewModelInstanceEnum, _>(|value| {
            value.base.view_model_property_id()
        }),
        Some(0),
        "native enum property ID"
    );
    value
}

fn native_property_named(view_model: &CoreHandle, name: &str) -> CoreHandle {
    let value = view_model
        .with_downcast::<ViewModelInstance, _>(|view_model| {
            view_model.property_value_named(name).or_else(|| {
                let definition = view_model.get_view_model()?;
                let property_id = definition.with_downcast::<ViewModel, _>(|definition| {
                    (0_u32..)
                        .map_while(|index| definition.property_at(index as usize))
                        .position(|property| {
                            property
                                .with(|property| {
                                    property
                                        .as_view_model_property()
                                        .is_some_and(|property| property.base.name() == name)
                                })
                                .unwrap_or(false)
                        })
                })??;
                view_model.property_value_by_id(property_id as u32)
            })
        })
        .flatten()
        .unwrap_or_else(|| panic!("native property {name}"));
    value
}

fn native_enum_property_named(view_model: &CoreHandle, name: &str) -> CoreHandle {
    let value = native_property_named(view_model, name);
    assert!(
        value
            .with_downcast::<ViewModelInstanceEnum, _>(|_| ())
            .is_some(),
        "native property {name} is enum"
    );
    value
}

fn native_enum_property_name_path(view_model: &CoreHandle, path: &str) -> CoreHandle {
    let mut segments = path.split('/').peekable();
    let mut instance = view_model.clone();
    loop {
        let name = segments.next().expect("non-empty native property path");
        if segments.peek().is_none() {
            return native_enum_property_named(&instance, name);
        }
        instance = native_property_named(&instance, name)
            .with_downcast::<ViewModelInstanceViewModel, _>(|value| {
                value.reference_view_model_instance()
            })
            .flatten()
            .unwrap_or_else(|| panic!("native nested view-model property {name}"));
    }
}

fn native_nested_enum_property(view_model: &CoreHandle) -> CoreHandle {
    let child = view_model
        .with_downcast::<ViewModelInstance, _>(|view_model| view_model.property_value_by_id(0))
        .flatten()
        .and_then(|value| {
            value.with_downcast::<ViewModelInstanceViewModel, _>(|value| {
                value.reference_view_model_instance()
            })
        })
        .flatten()
        .expect("native nested view-model instance");
    native_enum_property(&child)
}

fn set_native_enum_source(view_model: &CoreHandle, value: u64) {
    let property = native_enum_property(view_model);
    CoreRegistry::set_uint_handle(
        &property,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value as u32,
    );
    assert_eq!(default_enum_value(view_model), value);
}

fn number_source_value(property: &CoreHandle) -> f32 {
    property
        .with_downcast::<ViewModelInstanceNumber, _>(|value| value.base.property_value())
        .expect("native number value")
}

fn set_native_number_source(property: &CoreHandle, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        property,
        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value,
    ));
    assert_eq!(number_source_value(property), value);
}

fn compare_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    bound_view_model: Option<&CoreHandle>,
    label: &str,
) {
    compare_advance_at(cpp, rust, advanced, bound_view_model, 0, label);
}

fn compare_advance_at(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    bound_view_model: Option<&CoreHandle>,
    state_machine_index: usize,
    label: &str,
) {
    compare_advance_at_bindings(
        cpp,
        rust,
        advanced,
        bound_view_model,
        state_machine_index,
        &[0],
        label,
    );
}

fn compare_advance_at_bindings(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    bound_view_model: Option<&CoreHandle>,
    state_machine_index: usize,
    data_bind_indices: &[usize],
    label: &str,
) {
    compare_advance_state(cpp, rust, advanced, state_machine_index, label);

    for data_bind_index in data_bind_indices {
        let cpp_binding = cpp
            .enum_bindings
            .iter()
            .find(|binding| binding.data_bind_index == *data_bind_index)
            .unwrap_or_else(|| panic!("C++ enum binding {data_bind_index}"));
        let (source, target_value) = native_enum_binding_at(rust, *data_bind_index);
        let source_value = source.as_ref().map(enum_source_value);
        assert_eq!(
            cpp_binding.source_value, source_value,
            "{label} binding {data_bind_index} sourceValue"
        );
        assert_eq!(
            cpp_binding.target_value, target_value,
            "{label} binding {data_bind_index} targetValue"
        );
        if let Some(bound_source) = bound_view_model {
            assert_eq!(source.as_ref(), Some(bound_source));
        } else {
            assert!(source.is_none());
        }
    }
}

fn compare_advance_state(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    state_machine_index: usize,
    label: &str,
) {
    assert_eq!(
        cpp.state_machine_index, state_machine_index,
        "{label} stateMachineIndex"
    );
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

fn compare_number_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
    expected_source: &CoreHandle,
    label: &str,
) -> CoreHandle {
    let cpp_binding = cpp
        .number_bindings
        .iter()
        .find(|binding| binding.data_bind_index == data_bind_index)
        .unwrap_or_else(|| panic!("C++ number binding {data_bind_index}"));
    let (source, target, converter) = native_number_binding_at(rust, data_bind_index);
    assert_eq!(source.as_ref(), Some(expected_source));
    let source_value = source.as_ref().and_then(|source| {
        source.with_downcast::<ViewModelInstanceNumber, _>(|source| source.base.property_value())
    });
    match (cpp_binding.source_value, source_value) {
        (Some(cpp), Some(rust)) => assert_close(rust, cpp, label),
        (None, None) => {}
        (cpp, rust) => {
            panic!("{label} number binding {data_bind_index} source mismatch: {cpp:?} vs {rust:?}")
        }
    }
    match (cpp_binding.target_value, target) {
        (Some(cpp), Some(rust)) => assert_close(rust, cpp, label),
        (None, None) => {}
        (cpp, rust) => {
            panic!("{label} number binding {data_bind_index} target mismatch: {cpp:?} vs {rust:?}")
        }
    }
    converter.expect("native number converter occurrence")
}

fn assert_to_number_converter(converter: &CoreHandle, label: &str) {
    assert!(
        converter
            .with_downcast::<DataConverterToNumber, _>(|_| ())
            .is_some(),
        "{label} missing cloned DataConverterToNumber"
    );
}

fn compare_string_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    data_bind_index: usize,
    expected_source: Option<&CoreHandle>,
    label: &str,
) -> CoreHandle {
    let cpp_binding = cpp
        .string_bindings
        .iter()
        .find(|binding| binding.data_bind_index == data_bind_index)
        .unwrap_or_else(|| panic!("C++ string binding {data_bind_index}"));
    let (source, target, converter) = native_string_binding_at(rust, data_bind_index);
    assert_eq!(
        source.as_ref(),
        expected_source,
        "{label} string bind source"
    );
    assert_eq!(
        cpp_binding.source_value, None,
        "{label} enum source is not a string observation"
    );
    assert_eq!(
        cpp_binding.target_value, target,
        "{label} string binding {data_bind_index} targetValue"
    );
    converter.expect("native string converter occurrence")
}

fn assert_to_string_converter(converter: &CoreHandle, label: &str) {
    assert!(
        converter
            .with_downcast::<DataConverterToString, _>(|_| ())
            .is_some(),
        "{label} missing cloned DataConverterToString"
    );
}

fn assert_enum_operation_converter(converter: &CoreHandle, grouped: bool, label: &str) {
    if !grouped {
        assert_eq!(
            converter.with_downcast::<DataConverterOperationViewModel, _>(|converter| {
                converter.source_path_ids().to_vec()
            }),
            Some(vec![0, 1]),
            "{label} operation enum operand path"
        );
        return;
    }
    let items = converter
        .with_downcast::<DataConverterGroup, _>(|converter| converter.items().to_vec())
        .unwrap_or_else(|| panic!("{label} missing cloned DataConverterGroup"));
    let operation_paths = items
        .iter()
        .filter_map(|item| {
            item.with(|item| item.as_data_converter_group_item()?.converter())
                .flatten()
        })
        .filter_map(|converter| {
            converter.with_downcast::<DataConverterOperationViewModel, _>(|converter| {
                converter.source_path_ids().to_vec()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        operation_paths,
        [vec![0, 1]],
        "{label} grouped enum operand"
    );
}

fn compare_runtime_update(cpp_artboard: &CppArtboard, rust: &NativeFixture, label: &str) {
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_x = cpp_artboard
        .runtime_update
        .as_ref()
        .and_then(|update| {
            update
                .components
                .iter()
                .find(|component| component.local_id != 0 && component.local_transform.is_some())
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

fn compare_formula_number_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    expected_source: &CoreHandle,
    label: &str,
) {
    let cpp_binding = cpp
        .number_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .expect("C++ formula number binding 0");
    rust.with_instance(|rust| {
        let target = native_enum_target_for_instance_at(rust, 0);
        let bind = rust
            .bindable_data_bind_to_target(&target)
            .expect("native formula data-bind occurrence");
        let (source, converter) = bind
            .with(|bind| {
                let bind = bind.as_data_bind()?;
                Some((bind.source(), bind.converter()))
            })
            .flatten()
            .expect("native formula DataBind");
        assert_eq!(source.as_ref(), Some(expected_source));
        assert!(
            converter
                .and_then(|converter| {
                    converter.with_downcast::<DataConverterFormula, _>(|_| ())
                })
                .is_some(),
            "{label} missing native DataConverterFormula occurrence"
        );
        assert_eq!(cpp_binding.source_value, None, "{label} number sourceValue");
        let target_value = target
            .with_downcast::<BindablePropertyNumber, _>(|target| target.base.property_value());
        match (cpp_binding.target_value, target_value) {
            (Some(cpp), Some(rust)) => assert_close(rust, cpp, label),
            (None, None) => {}
            (cpp, rust) => panic!(
                "{label} formula number target presence mismatch: C++ {cpp:?}, Rust {rust:?}"
            ),
        }
    });
}

fn compare_formula_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    source: &CoreHandle,
    label: &str,
) {
    compare_advance_at_bindings(cpp, rust, advanced, Some(source), 0, &[1], label);
    compare_formula_number_binding(cpp, rust, source, label);
}

fn run_bind_source_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    bind: impl FnOnce(&NativeFixture) -> CoreHandle,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let seconds = [0.0, 0.0, 1.0];
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        seconds.len()
    );
    let mut bound_view_model = None;
    let mut bind = Some(bind);
    for (index, (cpp_advance, seconds)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(seconds)
        .enumerate()
    {
        if index == 1 {
            bound_view_model = Some(bind.take().expect("single native bind action")(&rust));
        }
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(
            cpp_advance,
            &rust.machine,
            advanced,
            bound_view_model.as_ref(),
            label,
        );
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

fn run_bound_two_advance_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    bind_and_mutate: impl FnOnce(&NativeFixture) -> CoreHandle,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let source = bind_and_mutate(&rust);
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
        compare_advance(cpp_advance, &rust.machine, advanced, Some(&source), label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_enum_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_bind_cpp.riv";
    let bytes = fixture_bytes(8374);
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
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        rust.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        native_enum_property(&rust.default_view_model)
    });
}

#[test]
fn state_machine_external_viewmodel_enum_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_external_viewmodel_enum_bind_cpp.riv";
    let bytes = fixture_bytes(8391);
    let forced_value = 1_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-enum".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        forced_value.to_string(),
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
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        set_native_enum_target(&rust.machine, forced_value);
        let external = rust
            ._file
            .with_file(|file| file.view_model_handle(0))
            .and_then(|view_model| {
                view_model.with_downcast::<ViewModel, _>(|view_model| view_model.instance_at(1))
            })
            .flatten()
            .expect("native external enum view-model instance");
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(external.clone()));
        native_enum_property(&external)
    });
}

#[test]
fn state_machine_owned_viewmodel_enum_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_enum_bind_cpp.riv";
    let bytes = fixture_bytes(8399);
    let value = 1_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-enum-state-machine-context".to_owned(),
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
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let owned = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(0)
                    .and_then(|view_model| file.create_view_model_instance(view_model))
            })
            .expect("native owned enum view-model instance");
        set_native_enum_source(&owned, value);
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        native_enum_property(&owned)
    });
}

#[test]
fn state_machine_owned_viewmodel_enum_source_handle_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_enum_source_handle_bind_cpp.riv";
    let bytes = fixture_bytes(8766);
    let value = 1_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-enum-state-machine-context".to_owned(),
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
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let owned = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(0)
                    .and_then(|view_model| file.create_view_model_instance(view_model))
            })
            .expect("native owned enum view-model instance");
        let source_handle = native_enum_property(&owned);
        CoreRegistry::set_uint_handle(
            &source_handle,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            value as u32,
        );
        assert_eq!(default_enum_value(&owned), value);
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        source_handle
    });
}

#[test]
fn state_machine_default_viewmodel_enum_source_handle_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_enum_source_handle_mutation_cpp.riv";
    let bytes = fixture_bytes(8745);
    let value = 0_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "choice".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let source_handle = native_enum_property(&rust.default_view_model);
        rust.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        CoreRegistry::set_uint_handle(
            &source_handle,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            value as u32,
        );
        assert_eq!(default_enum_value(&rust.default_view_model), value);
        source_handle
    });
}

#[test]
fn state_machine_default_viewmodel_nested_enum_source_handle_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_nested_enum_source_handle_cpp.riv";
    let bytes = nested_fixture_bytes(8777, 1, 1);
    let value = 0_u64;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-enum".to_owned(),
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
    run_bound_two_advance_case(&probe, label, &bytes, &args, |rust| {
        let source = native_nested_enum_property(&rust.default_view_model);
        rust.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        CoreRegistry::set_uint_handle(
            &source,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            value as u32,
        );
        assert_eq!(enum_source_value(&source), value);
        source
    });
}

fn run_formula_enum_context_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    bind: impl FnOnce(&NativeFixture) -> CoreHandle,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let source = bind(&rust);
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
        compare_formula_advance(cpp_advance, &rust.machine, advanced, &source, label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

fn create_owned_enum_context(rust: &NativeFixture, value: u64) -> (CoreHandle, CoreHandle) {
    let owned = rust
        ._file
        .with_file(|file| {
            file.view_model_handle(0)
                .and_then(|view_model| file.create_view_model_instance(view_model))
        })
        .expect("native owned enum view-model instance");
    let source = native_enum_property(&owned);
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        value as u32,
    );
    assert_eq!(enum_source_value(&source), value);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
    (owned, source)
}

#[test]
fn state_machine_imported_viewmodel_enum_formula_context_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_enum_formula_context_cpp.riv";
    let bytes = formula_enum_fixture_bytes(8738);
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-view-model-instance-source-enum".to_owned(),
        "0".to_owned(),
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
    run_formula_enum_context_case(&probe, label, &bytes, &args, |rust| {
        let source = native_enum_property(&rust.default_view_model);
        rust.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        CoreRegistry::set_uint_handle(
            &source,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            0,
        );
        assert_eq!(enum_source_value(&source), 0);
        source
    });
}

#[test]
fn state_machine_owned_viewmodel_enum_formula_context_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_enum_formula_context_cpp.riv";
    let bytes = formula_enum_fixture_bytes(8739);
    let args = [
        "--runtime-bind-owned-view-model-enum-state-machine-context".to_owned(),
        "0".to_owned(),
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
    run_formula_enum_context_case(&probe, label, &bytes, &args, |rust| {
        let (_owned, source) = create_owned_enum_context(rust, 1);
        source
    });
}

#[test]
fn state_machine_owned_viewmodel_enum_formula_source_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_enum_formula_source_mutation_cpp.riv";
    let bytes = formula_enum_fixture_bytes(9310);
    let args = [
        "--runtime-bind-owned-view-model-enum-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-owned-view-model-source-enum".to_owned(),
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
    let rust = native_fixture(&bytes, label);
    let (_owned, source) = create_owned_enum_context(&rust, 1);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_formula_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        &source,
        label,
    );
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        0,
    );
    assert_eq!(enum_source_value(&source), 0);
    for (step, (cpp_advance, seconds)) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
        .enumerate()
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        let step_label = format!("{label} action {}", step + 1);
        compare_formula_advance(cpp_advance, &rust.machine, advanced, &source, &step_label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_owned_viewmodel_nested_enum_source_handle_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_enum_source_handle_bind_cpp.riv";
    let bytes = nested_fixture_bytes(8788, 0, 1);
    let value = 1_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-enum-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/choice".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let owned = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(0)
                    .and_then(|view_model| file.create_view_model_instance(view_model))
            })
            .expect("native owned nested enum view-model instance");
        let source = native_nested_enum_property(&owned);
        CoreRegistry::set_uint_handle(
            &source,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            value as u32,
        );
        assert_eq!(enum_source_value(&source), value);
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
        source
    });
}

fn run_imported_shared_source_handle_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    nested: bool,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let machine_b = rust
        .artboard
        .state_machine_at(1)
        .expect("second native state machine");
    let source = if nested {
        native_nested_enum_property(&rust.default_view_model)
    } else {
        native_enum_property(&rust.default_view_model)
    };
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        0,
    );
    assert_eq!(enum_source_value(&source), 0);

    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let machines = [&rust.machine, &machine_b];
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    for (index, (cpp_advance, machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(machines)
        .enumerate()
    {
        machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        let advanced = machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        compare_advance_at(cpp_advance, machine, advanced, Some(&source), index, label);
    }
}

#[test]
fn state_machine_imported_viewmodel_enum_source_handle_mutation_is_shared_across_state_machines_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_enum_source_handle_mutation_shared_cpp.riv";
    let bytes = fixture_bytes_with_state_machines(8711, 2);
    let args = [
        "--complete-view-model-properties".to_owned(),
        "--runtime-set-view-model-instance-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "choice".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
    ];
    run_imported_shared_source_handle_case(&probe, label, &bytes, &args, false);
}

#[test]
fn state_machine_imported_viewmodel_nested_enum_source_handle_mutation_is_shared_across_state_machines_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_nested_enum_source_handle_mutation_shared_cpp.riv";
    let bytes = nested_fixture_bytes(8712, 1, 2);
    let args = [
        "--complete-view-model-properties".to_owned(),
        "--runtime-set-view-model-instance-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/choice".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
    ];
    run_imported_shared_source_handle_case(&probe, label, &bytes, &args, true);
}

#[test]
fn state_machine_default_viewmodel_enum_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_target_to_source_cpp.riv";
    let bytes =
        target_to_source_fixture_bytes(8479, 0, &[DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY, 0]);
    let forced_value = 1_u64;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-enum".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        forced_value.to_string(),
        "--runtime-advance-state-machine-data-context".to_owned(),
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
    let source = native_enum_property(&rust.default_view_model);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 5);

    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        None,
        0,
        &[0, 1],
        label,
    );

    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    rust.machine
        .with_instance_mut(|machine| machine.advanced_data_context());
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        Some(&source),
        0,
        &[0, 1],
        label,
    );

    set_native_enum_target_at(&rust.machine, 0, forced_value);
    rust.machine
        .with_instance_mut(|machine| machine.advanced_data_context());
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[2],
        &rust.machine,
        false,
        Some(&source),
        0,
        &[0, 1],
        label,
    );

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[3..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance_at_bindings(
            cpp_advance,
            &rust.machine,
            advanced,
            Some(&source),
            0,
            &[0, 1],
            label,
        );
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

fn run_enum_public_update_target_to_source_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    data_bind_indices: &[usize],
) {
    let forced_value = 0_u64;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-enum".to_owned(),
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
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, &args);
    let rust = native_fixture(bytes, label);
    let source = native_enum_property(&rust.default_view_model);
    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        Some(&source),
        0,
        data_bind_indices,
        label,
    );

    set_native_enum_target_at(&rust.machine, 0, forced_value);
    DataBindContainerOwner::StateMachine(rust.machine.downgrade()).update_data_binds(true);
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        Some(&source),
        0,
        data_bind_indices,
        label,
    );

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance_at_bindings(
            cpp_advance,
            &rust.machine,
            advanced,
            Some(&source),
            0,
            data_bind_indices,
            label,
        );
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn enum_public_update_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_public_update_target_to_source_cpp.riv";
    let bytes = target_to_source_fixture_bytes(8655, 1, &[DATA_BIND_TWO_WAY]);
    run_enum_public_update_target_to_source_case(&probe, label, &bytes, &[0]);
}

#[test]
fn enum_public_update_observer_preserves_target_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_public_update_observer_preserves_target_cpp.riv";
    let bytes = target_to_source_fixture_bytes(8656, 1, &[DATA_BIND_TWO_WAY, 0]);
    run_enum_public_update_target_to_source_case(&probe, label, &bytes, &[0, 1]);
}

fn run_default_enum_source_mutation_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    data_bind_indices: &[usize],
    source: impl FnOnce(&NativeFixture) -> CoreHandle,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_at_bindings(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        None,
        0,
        data_bind_indices,
        label,
    );

    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    let source = source(&rust);
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        0,
    );
    assert_eq!(enum_source_value(&source), 0);

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance_at_bindings(
            cpp_advance,
            &rust.machine,
            advanced,
            Some(&source),
            0,
            data_bind_indices,
            label,
        );
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_enum_source_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_source_mutation_cpp.riv";
    let bytes = fixture_bytes(8383);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-enum".to_owned(),
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
    run_default_enum_source_mutation_case(&probe, label, &bytes, &args, &[0], |rust| {
        native_enum_property(&rust.default_view_model)
    });
}

#[test]
fn state_machine_default_viewmodel_enum_source_mutation_observer_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_enum_source_mutation_observer_cpp.riv";
    let bytes = target_to_source_fixture_bytes(8670, 1, &[DATA_BIND_TWO_WAY, 0]);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-enum".to_owned(),
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
    run_default_enum_source_mutation_case(&probe, label, &bytes, &args, &[0, 1], |rust| {
        native_enum_property(&rust.default_view_model)
    });
}

#[test]
fn state_machine_default_viewmodel_enum_source_name_mutation_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_enum_source_name_mutation_cpp.riv";
    let bytes = fixture_bytes(8627);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "choice".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_default_enum_source_mutation_case(&probe, label, &bytes, &args, &[0], |rust| {
        native_enum_property_named(&rust.default_view_model, "choice")
    });
}

#[test]
fn state_machine_imported_viewmodel_enum_source_mutation_is_shared_across_state_machines_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_imported_viewmodel_enum_source_mutation_shared_cpp.riv";
    let bytes = fixture_bytes_with_state_machines(8605, 2);
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-view-model-instance-source-enum".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_fixture(&bytes, label);
    let machine_b = rust
        .artboard
        .state_machine_at(1)
        .expect("second native state machine");
    let source = native_enum_property(&rust.default_view_model);
    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        0,
    );
    assert_eq!(enum_source_value(&source), 0);

    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_at(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        Some(&source),
        0,
        label,
    );
    machine_b.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    let advanced = machine_b.with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_at(
        &cpp_artboard.runtime_state_machine_advances[1],
        &machine_b,
        advanced,
        Some(&source),
        1,
        label,
    );
}

fn run_imported_shared_source_name_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    args: &[String],
    path: &str,
) {
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, args);
    let rust = native_fixture(bytes, label);
    let machine_b = rust
        .artboard
        .state_machine_at(1)
        .expect("second native state machine");
    rust._file
        .complete_view_model_properties(&rust.default_view_model);
    let source = native_enum_property_name_path(&rust.default_view_model, path);
    CoreRegistry::set_uint_handle(
        &source,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
        0,
    );
    assert_eq!(enum_source_value(&source), 0);

    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    let machines = [&rust.machine, &machine_b];
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);
    for (index, (cpp_advance, machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(machines)
        .enumerate()
    {
        machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        let advanced = machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        compare_advance_at(cpp_advance, machine, advanced, Some(&source), index, label);
    }
}

#[test]
fn state_machine_imported_viewmodel_enum_source_name_mutation_is_shared_across_state_machines_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_enum_source_name_mutation_shared_cpp.riv";
    let bytes = fixture_bytes_with_state_machines(8616, 2);
    let args = [
        "--complete-view-model-properties".to_owned(),
        "--runtime-set-view-model-instance-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "choice".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
    ];
    run_imported_shared_source_name_case(&probe, label, &bytes, &args, "choice");
}

#[test]
fn state_machine_imported_viewmodel_nested_enum_source_name_path_mutation_is_shared_across_state_machines_matches_cpp_probe()
 {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_imported_viewmodel_nested_enum_source_name_path_mutation_shared_cpp.riv";
    let bytes = nested_fixture_bytes(8692, 1, 2);
    let args = [
        "--complete-view-model-properties".to_owned(),
        "--runtime-set-view-model-instance-source-enum-by-name".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/choice".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
    ];
    run_imported_shared_source_name_case(&probe, label, &bytes, &args, "child/choice");
}

#[test]
fn state_machine_default_viewmodel_nested_enum_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_default_viewmodel_nested_enum_bind_cpp.riv";
    let bytes = nested_fixture_bytes(8586, 1, 1);
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
    run_bound_two_advance_case(&probe, label, &bytes, &args, |rust| {
        let source = native_nested_enum_property(&rust.default_view_model);
        rust.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(rust.default_view_model.clone())
        });
        source
    });
}

#[test]
fn state_machine_owned_viewmodel_nested_enum_name_path_bind_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_nested_enum_name_path_bind_cpp.riv";
    let bytes = nested_fixture_bytes(8582, 0, 1);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-enum-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/choice".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let owned = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(0)
                    .and_then(|view_model| file.create_view_model_instance(view_model))
            })
            .expect("native owned nested enum view-model instance");
        let source = native_enum_property_name_path(&owned, "child/choice");
        CoreRegistry::set_uint_handle(
            &source,
            ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
            1,
        );
        assert_eq!(enum_source_value(&source), 1);
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
        source
    });
}

#[test]
fn state_machine_owned_viewmodel_imported_intermediate_enum_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_owned_viewmodel_imported_intermediate_enum_source_cpp.riv";
    let bytes = nested_fixture_bytes(8592, 1, 1);
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
    run_bind_source_case(&probe, label, &bytes, &args, |rust| {
        let owned = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(0)
                    .and_then(|view_model| file.create_view_model_instance(view_model))
            })
            .expect("native owned root view-model instance");
        let imported_child = rust
            ._file
            .with_file(|file| {
                file.view_model_handle(1).and_then(|view_model| {
                    view_model
                        .with_downcast::<ViewModel, _>(|view_model| view_model.instance_at(0))
                        .flatten()
                })
            })
            .expect("native imported child view-model instance");
        let child_property = native_property_named(&owned, "child");
        assert!(
            ViewModelInstance::replace_view_model_property_occurrence(
                &owned,
                &child_property,
                Some(imported_child.clone()),
            ),
            "{label} imported child replacement"
        );
        let source = native_enum_property(&imported_child);
        assert_eq!(enum_source_value(&source), 1);
        rust.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
        source
    });
}

fn run_enum_to_number_target_dirty_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    public_update: bool,
) {
    let mut args = vec![
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "4.46".to_owned(),
    ];
    args.push(
        if public_update {
            "--runtime-update-state-machine-data-binds"
        } else {
            "--runtime-advance-state-machine-data-context"
        }
        .to_owned(),
    );
    args.push("0".to_owned());
    args.extend(
        [
            "--runtime-advance-state-machine",
            "0",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "1",
        ]
        .map(str::to_owned),
    );

    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, &args);
    let rust = native_fixture(bytes, label);
    let source = native_enum_property(&rust.default_view_model);
    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_state(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        0,
        label,
    );
    let converter = compare_number_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        0,
        &source,
        label,
    );
    assert_to_number_converter(&converter, label);

    set_native_number_target(&rust.machine, 0, 4.46);
    if public_update {
        DataBindContainerOwner::StateMachine(rust.machine.downgrade()).update_data_binds(true);
    } else {
        rust.machine
            .with_instance_mut(|machine| machine.advanced_data_context());
    }
    compare_advance_state(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        0,
        label,
    );
    let converter = compare_number_binding(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        0,
        &source,
        label,
    );
    assert_to_number_converter(&converter, label);

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance_state(cpp_advance, &rust.machine, advanced, 0, label);
        let converter = compare_number_binding(cpp_advance, &rust.machine, 0, &source, label);
        assert_to_number_converter(&converter, label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn to_number_enum_public_update_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_enum_to_number_public_update_cpp.riv";
    let bytes = enum_to_number_fixture_bytes(8536, DATA_BIND_TWO_WAY);
    run_enum_to_number_target_dirty_case(&probe, label, &bytes, true);
}

#[test]
fn to_number_enum_main_to_target_two_way_target_dirty_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_to_number_main_to_target_two_way_target_dirty_cpp.riv";
    let bytes = enum_to_number_fixture_bytes(8506, DATA_BIND_TWO_WAY);
    run_enum_to_number_target_dirty_case(&probe, label, &bytes, false);
}

fn run_owned_enum_operation_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    grouped: bool,
) {
    let args = [
        "--runtime-bind-owned-view-model-number-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.4".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-owned-view-model-source-enum".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, &args);
    let rust = native_fixture(bytes, label);
    let owned = rust
        ._file
        .with_file(|file| {
            file.view_model_handle(0)
                .and_then(|view_model| file.create_view_model_instance(view_model))
        })
        .expect("native owned operation view-model instance");
    let amount = native_property_named(&owned, "amount");
    let choice = native_enum_property_named(&owned, "choice");
    set_native_number_source(&amount, 0.4);
    rust.machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owned));
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    for (step, (cpp_advance, seconds)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 0.0, 1.0])
        .enumerate()
    {
        if step == 1 {
            CoreRegistry::set_uint_handle(
                &choice,
                ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
                1,
            );
            assert_eq!(enum_source_value(&choice), 1);
        }
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        let step_label = format!("{label} action {step}");
        compare_advance_at_bindings(
            cpp_advance,
            &rust.machine,
            advanced,
            Some(&choice),
            0,
            &[1],
            &step_label,
        );
        let converter = compare_number_binding(cpp_advance, &rust.machine, 0, &amount, &step_label);
        assert_enum_operation_converter(&converter, grouped, &step_label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn operation_viewmodel_owned_enum_source_mutation_fallback_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_operation_viewmodel_enum_source_mutation_cpp.riv";
    let bytes = enum_operation_fixture_bytes(9321, false);
    run_owned_enum_operation_case(&probe, label, &bytes, false);
}

#[test]
fn operation_viewmodel_group_owned_enum_source_mutation_fallback_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label = "synthetic/runtime_state_machine_owned_viewmodel_number_operation_viewmodel_group_enum_source_mutation_cpp.riv";
    let bytes = enum_operation_fixture_bytes(9322, true);
    run_owned_enum_operation_case(&probe, label, &bytes, true);
}

enum EnumToStringTargetFlow {
    PublicUpdate,
    MainToSource,
    MainToTarget,
}

fn run_enum_to_string_target_case(
    probe: &std::path::Path,
    label: &str,
    bytes: &[u8],
    flow: EnumToStringTargetFlow,
) {
    let mut args = vec![
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
    ];
    match flow {
        EnumToStringTargetFlow::MainToSource => {
            args.extend(["--runtime-advance-state-machine-data-context", "0"].map(str::to_owned))
        }
        EnumToStringTargetFlow::PublicUpdate | EnumToStringTargetFlow::MainToTarget => {
            args.extend(["--runtime-advance-state-machine", "0", "0"].map(str::to_owned))
        }
    }
    args.extend(
        [
            "--runtime-set-state-machine-bindable-string",
            "0",
            "0",
            "manual",
        ]
        .map(str::to_owned),
    );
    args.extend(
        match flow {
            EnumToStringTargetFlow::PublicUpdate => {
                ["--runtime-update-state-machine-data-binds", "0"]
            }
            EnumToStringTargetFlow::MainToSource | EnumToStringTargetFlow::MainToTarget => {
                ["--runtime-advance-state-machine-data-context", "0"]
            }
        }
        .map(str::to_owned),
    );
    args.extend(
        [
            "--runtime-advance-state-machine",
            "0",
            "0",
            "--runtime-advance-state-machine",
            "0",
            "1",
        ]
        .map(str::to_owned),
    );

    let cpp = read_cpp_probe_bytes_with_args(probe, label, bytes, &args);
    let rust = native_fixture(bytes, label);
    let source = native_enum_property(&rust.default_view_model);
    rust.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(rust.default_view_model.clone())
    });
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    let first_advanced = match flow {
        EnumToStringTargetFlow::MainToSource => {
            rust.machine
                .with_instance_mut(|machine| machine.advanced_data_context());
            false
        }
        EnumToStringTargetFlow::PublicUpdate | EnumToStringTargetFlow::MainToTarget => rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(0.0)),
    };
    compare_advance_state(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        first_advanced,
        0,
        label,
    );
    let converter = compare_string_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        0,
        Some(&source),
        label,
    );
    assert_to_string_converter(&converter, label);

    set_native_string_target(&rust.machine, 0, "manual");
    match flow {
        EnumToStringTargetFlow::PublicUpdate => {
            DataBindContainerOwner::StateMachine(rust.machine.downgrade()).update_data_binds(true)
        }
        EnumToStringTargetFlow::MainToSource | EnumToStringTargetFlow::MainToTarget => {
            rust.machine
                .with_instance_mut(|machine| machine.advanced_data_context());
        }
    }
    compare_advance_state(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        0,
        label,
    );
    let converter = compare_string_binding(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        0,
        Some(&source),
        label,
    );
    assert_to_string_converter(&converter, label);

    for (step, (cpp_advance, seconds)) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
        .enumerate()
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        let step_label = format!("{label} action {}", step + 2);
        compare_advance_state(cpp_advance, &rust.machine, advanced, 0, &step_label);
        let converter =
            compare_string_binding(cpp_advance, &rust.machine, 0, Some(&source), &step_label);
        assert_to_string_converter(&converter, &step_label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}

#[test]
fn enum_to_string_public_update_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_to_string_public_update_target_to_source_cpp.riv";
    let bytes = enum_to_string_fixture_bytes(8548, DATA_BIND_TWO_WAY);
    run_enum_to_string_target_case(&probe, label, &bytes, EnumToStringTargetFlow::PublicUpdate);
}

#[test]
fn enum_to_string_main_to_source_two_way_target_to_source_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_to_string_main_to_source_two_way_target_to_source_cpp.riv";
    let bytes = enum_to_string_fixture_bytes(8558, DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY);
    run_enum_to_string_target_case(&probe, label, &bytes, EnumToStringTargetFlow::MainToSource);
}

#[test]
fn enum_to_string_main_to_target_two_way_target_dirty_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let label = "synthetic/runtime_state_machine_default_viewmodel_enum_to_string_main_to_target_two_way_target_dirty_cpp.riv";
    let bytes = enum_to_string_fixture_bytes(8517, DATA_BIND_TWO_WAY);
    run_enum_to_string_target_case(&probe, label, &bytes, EnumToStringTargetFlow::MainToTarget);
}

#[test]
fn state_machine_default_viewmodel_enum_to_string_converter_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let label =
        "synthetic/runtime_state_machine_default_viewmodel_enum_to_string_converter_cpp.riv";
    let bytes = enum_to_string_fixture_bytes(8418, 0);
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
    let rust = native_fixture(&bytes, label);
    let source = native_enum_property(&rust.default_view_model);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    for (step, (cpp_advance, seconds)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 0.0, 1.0])
        .enumerate()
    {
        if step == 1 {
            rust.machine.with_instance_mut(|machine| {
                machine.bind_view_model_instance(rust.default_view_model.clone())
            });
        }
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        let step_label = format!("{label} action {step}");
        compare_advance_state(cpp_advance, &rust.machine, advanced, 0, &step_label);
        let expected_source = (step != 0).then_some(&source);
        let converter =
            compare_string_binding(cpp_advance, &rust.machine, 0, expected_source, &step_label);
        assert_to_string_converter(&converter, &step_label);
    }
    compare_runtime_update(cpp_artboard, &rust, label);
}
