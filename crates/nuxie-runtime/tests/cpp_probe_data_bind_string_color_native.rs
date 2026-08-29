//! String/color data-bind differentials observed directly from live native owners.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        state_machine::StateMachine,
        state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
    },
    artboard::Artboard as NativeArtboard,
    data_bind::{
        bindable_property_color::BindablePropertyColor,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        converters::data_converter_formula::DataConverterFormula,
        converters::data_converter_group::DataConverterGroup,
        converters::data_converter_string_pad::DataConverterStringPad,
        converters::data_converter_string_remove_zeros::DataConverterStringRemoveZeros,
        converters::data_converter_string_trim::DataConverterStringTrim,
        converters::data_converter_to_string::DataConverterToString, data_bind::DataBind,
        data_bind_container::DataBindContainerOwner, data_bind_context::DataBindContext,
    },
    math::random::RandomProvider,
    node::Node,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use nuxie_runtime::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};
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
    view_model_triggers: Vec<CppRuntimeStateMachineViewModelTrigger>,
    #[serde(rename = "randomTotalCalls", default)]
    random_total_calls: usize,
    #[serde(rename = "numberBindings", default)]
    number_bindings: Vec<CppRuntimeStateMachineNumberBinding>,
    #[serde(rename = "stringBindings", default)]
    string_bindings: Vec<CppRuntimeStateMachineStringBinding>,
    #[serde(rename = "colorBindings", default)]
    color_bindings: Vec<CppRuntimeStateMachineColorBinding>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineViewModelTrigger {
    index: usize,
    #[serde(rename = "viewModelPropertyId")]
    view_model_property_id: u32,
    value: u32,
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
struct CppRuntimeStateMachineStringBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "sourceValue")]
    source_value: Option<String>,
    #[serde(rename = "targetValue")]
    target_value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineColorBinding {
    #[serde(rename = "dataBindIndex")]
    data_bind_index: usize,
    #[serde(rename = "sourceValue")]
    source_value: Option<u32>,
    #[serde(rename = "targetValue")]
    target_value: Option<u32>,
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
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
}

static RANDOM_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

fn counted_runtime_random_probe_args(values: &[f32], extra_args: &[String]) -> Vec<String> {
    let mut args = vec!["--runtime-random-reset".to_owned()];
    for value in values {
        args.push("--runtime-random-value".to_owned());
        args.push(value.to_string());
    }
    args.extend_from_slice(extra_args);
    args
}

impl NativeFixture {
    fn bind_default_view_model(&self, label: &str) -> nuxie_runtime::CoreHandle {
        let view_model = self.external_view_model_instance(0, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
        view_model
    }

    fn external_view_model_instance(
        &self,
        instance_index: usize,
        label: &str,
    ) -> nuxie_runtime::CoreHandle {
        let view_model = self
            .file
            .with_file(|file| file.view_model(0))
            .unwrap_or_else(|| panic!("missing native view model for {label}"));
        view_model
            .with_downcast::<ViewModel, _>(|view_model| view_model.instance_at(instance_index))
            .flatten()
            .unwrap_or_else(|| {
                panic!("missing native view-model instance {instance_index} for {label}")
            })
    }

    fn bind_external_view_model(
        &self,
        instance_index: usize,
        label: &str,
    ) -> nuxie_runtime::CoreHandle {
        let view_model = self.external_view_model_instance(instance_index, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
        view_model
    }

    fn create_owned_view_model(&self, label: &str) -> nuxie_runtime::CoreHandle {
        let view_model = self
            .file
            .with_file(|file| file.view_model(0))
            .unwrap_or_else(|| panic!("missing native view model for {label}"));
        self.file
            .with_file(|file| file.create_view_model_instance(view_model))
            .unwrap_or_else(|| panic!("missing native owned view-model instance for {label}"))
    }

    fn default_view_model_property(
        &self,
        instance: &nuxie_runtime::CoreHandle,
        property_name: &str,
        label: &str,
    ) -> nuxie_runtime::CoreHandle {
        let view_model = self
            .file
            .with_file(|file| file.view_model(0))
            .unwrap_or_else(|| panic!("missing native view model for {label}"));
        let property_id = view_model
            .with_downcast::<ViewModel, _>(|view_model| {
                view_model.properties().iter().position(|property| {
                    property
                        .with(|property| {
                            property
                                .as_view_model_property()
                                .is_some_and(|property| property.base.name() == property_name)
                        })
                        .unwrap_or(false)
                })
            })
            .flatten()
            .unwrap_or_else(|| {
                panic!("missing native schema property {property_name} for {label}")
            });
        instance
            .with(|instance| {
                let instance = instance
                    .as_view_model_instance()
                    .expect("native ViewModelInstance");
                instance
                    .property_value_named(property_name)
                    .or_else(|| instance.property_value_by_id(property_id as u32))
            })
            .flatten()
            .unwrap_or_else(|| panic!("missing native property {property_name} for {label}"))
    }

    fn set_default_string_property(
        &self,
        instance: &nuxie_runtime::CoreHandle,
        property_name: &str,
        value: &str,
        label: &str,
    ) {
        self.default_view_model_property(instance, property_name, label)
            .with_downcast_mut::<ViewModelInstanceString, _>(|property| property.set_value(value))
            .unwrap_or_else(|| panic!("wrong native string property type for {label}"));
    }

    fn set_default_color_property(
        &self,
        instance: &nuxie_runtime::CoreHandle,
        property_name: &str,
        value: u32,
        label: &str,
    ) {
        self.default_view_model_property(instance, property_name, label)
            .with_downcast_mut::<ViewModelInstanceColor, _>(|property| {
                property.set_value(value as i32)
            })
            .unwrap_or_else(|| panic!("wrong native color property type for {label}"));
    }

    fn bind_owned_string(&self, value: &str, label: &str) -> nuxie_runtime::CoreHandle {
        let owned = self.create_owned_view_model(label);
        set_view_model_string_property(&owned, "label", value, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        owned
    }

    fn bind_owned_color(&self, value: u32, label: &str) -> nuxie_runtime::CoreHandle {
        let owned = self.create_owned_view_model(label);
        set_view_model_color_property(&owned, "tint", value, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        owned
    }

    fn bind_owned_nested_string(&self, value: &str, label: &str) -> nuxie_runtime::CoreHandle {
        let owned = self.create_owned_view_model(label);
        let child = named_view_model_property(&owned, "child", label)
            .with_downcast::<ViewModelInstanceViewModel, _>(|property| {
                property.reference_view_model_instance()
            })
            .flatten()
            .unwrap_or_else(|| panic!("missing generated child view-model instance for {label}"));
        set_view_model_string_property(&child, "label", value, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        owned
    }

    fn bind_owned_nested_color(&self, value: u32, label: &str) -> nuxie_runtime::CoreHandle {
        let owned = self.create_owned_view_model(label);
        let child = named_view_model_property(&owned, "child", label)
            .with_downcast::<ViewModelInstanceViewModel, _>(|property| {
                property.reference_view_model_instance()
            })
            .flatten()
            .unwrap_or_else(|| panic!("missing generated child view-model instance for {label}"));
        set_view_model_color_property(&child, "tint", value, label);
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        owned
    }

    fn bind_owned_imported_child(&self, label: &str) -> nuxie_runtime::CoreHandle {
        let owned = self.create_owned_view_model(label);
        let child_model = self
            .file
            .with_file(|file| file.view_model(1))
            .unwrap_or_else(|| panic!("missing native child view model for {label}"));
        let imported = child_model
            .with_downcast::<ViewModel, _>(|view_model| view_model.instance_at(0))
            .flatten()
            .unwrap_or_else(|| panic!("missing native imported child instance for {label}"));
        assert!(
            ViewModelInstance::replace_view_model_by_name(&owned, "child", imported),
            "failed to replace generated child with imported child for {label}"
        );
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
        owned
    }

    fn set_bindable_number(&self, value: f32, label: &str) {
        self.machine.with_instance_mut(|machine| {
            let target = bindable_property(machine, label);
            target
                .with_downcast_mut::<BindablePropertyNumber, _>(|property| {
                    let mut base = std::mem::take(&mut property.base);
                    base.set_property_value(value, property);
                    property.base = base;
                })
                .unwrap_or_else(|| panic!("wrong native bindable number type for {label}"));
            if let Some(source_bind) = machine.bindable_data_bind_to_source(&target) {
                DataBind::update_source_binding_handle(&source_bind, true);
                DataBindContainerOwner::StateMachine(self.machine.downgrade())
                    .update_data_binds(false);
            }
        });
    }

    fn set_bindable_string(&self, value: &str, label: &str) {
        self.machine.with_instance_mut(|machine| {
            let target = bindable_property(machine, label);
            target
                .with_downcast_mut::<BindablePropertyString, _>(|property| {
                    let mut base = std::mem::take(&mut property.base);
                    base.set_property_value(value.to_owned(), property);
                    property.base = base;
                })
                .unwrap_or_else(|| panic!("wrong native bindable string type for {label}"));
        });
    }

    fn set_bindable_color(&self, value: u32, label: &str) {
        self.machine.with_instance_mut(|machine| {
            let target = bindable_property(machine, label);
            target
                .with_downcast_mut::<BindablePropertyColor, _>(|property| {
                    let mut base = std::mem::take(&mut property.base);
                    base.set_property_value(value as i32, property);
                    property.base = base;
                })
                .unwrap_or_else(|| panic!("wrong native bindable color type for {label}"));
        });
    }
}

fn named_view_model_property(
    instance: &nuxie_runtime::CoreHandle,
    property_name: &str,
    label: &str,
) -> nuxie_runtime::CoreHandle {
    instance
        .with(|instance| {
            instance
                .as_view_model_instance()
                .expect("native ViewModelInstance")
                .property_value_named(property_name)
        })
        .flatten()
        .unwrap_or_else(|| panic!("missing native property {property_name} for {label}"))
}

fn set_view_model_string_property(
    instance: &nuxie_runtime::CoreHandle,
    property_name: &str,
    value: &str,
    label: &str,
) {
    named_view_model_property(instance, property_name, label)
        .with_downcast_mut::<ViewModelInstanceString, _>(|property| property.set_value(value))
        .unwrap_or_else(|| panic!("wrong native string property type for {label}"));
}

fn set_view_model_color_property(
    instance: &nuxie_runtime::CoreHandle,
    property_name: &str,
    value: u32,
    label: &str,
) {
    named_view_model_property(instance, property_name, label)
        .with_downcast_mut::<ViewModelInstanceColor, _>(|property| property.set_value(value as i32))
        .unwrap_or_else(|| panic!("wrong native color property type for {label}"));
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
    NativeFixture {
        file,
        artboard,
        machine,
    }
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    push_var_uint(
        bytes,
        u64::from(property_key_for_name(type_name, property_name)),
    );
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_color_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    push_var_uint(
        bytes,
        u64::from(property_key_for_name(type_name, property_name)),
    );
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_bytes_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &[u8]) {
    push_var_uint(
        bytes,
        u64::from(property_key_for_name(type_name, property_name)),
    );
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_source_path(bytes: &mut Vec<u8>, property_type: &str, path: &[u32]) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(property_type, "propertyValue")),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
    });
}

fn push_bindable_string_data_bind_context(bytes: &mut Vec<u8>, value: &str, path: &[u32]) {
    push_object_with_properties(bytes, "BindablePropertyString", |bytes| {
        push_string_property(bytes, "BindablePropertyString", "propertyValue", value);
    });
    push_source_path(bytes, "BindablePropertyString", path);
}

fn push_bindable_color_data_bind_context(bytes: &mut Vec<u8>, value: u32, path: &[u32]) {
    push_object_with_properties(bytes, "BindablePropertyColor", |bytes| {
        push_color_property(bytes, "BindablePropertyColor", "propertyValue", value);
    });
    push_source_path(bytes, "BindablePropertyColor", path);
}

fn push_bindable_string_data_bind_context_with_converter_and_flags(
    bytes: &mut Vec<u8>,
    value: &str,
    path: &[u32],
    converter_id: Option<u64>,
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
        if let Some(converter_id) = converter_id {
            push_uint_property(bytes, "DataBindContext", "converterId", converter_id);
        }
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn push_bindable_color_data_bind_context_with_flags(
    bytes: &mut Vec<u8>,
    value: u32,
    path: &[u32],
    flags: u64,
) {
    let mut source_path_ids = Vec::new();
    for path_id in path {
        push_var_uint(&mut source_path_ids, u64::from(*path_id));
    }
    push_object_with_properties(bytes, "BindablePropertyColor", |bytes| {
        push_color_property(bytes, "BindablePropertyColor", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBindContext", |bytes| {
        push_uint_property(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key_for_name(
                "BindablePropertyColor",
                "propertyValue",
            )),
        );
        push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path_ids);
        if flags != 0 {
            push_uint_property(bytes, "DataBindContext", "flags", flags);
        }
    });
}

fn push_state_machine(bytes: &mut Vec<u8>, push_data_bind: impl FnOnce(&mut Vec<u8>)) {
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
    push_data_bind(bytes);
    push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
        push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
    });
    push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
}

fn string_fixture_bytes(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "ready");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "alternate");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "blocked");
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_state_machine(bytes, |bytes| {
            push_bindable_string_data_bind_context(bytes, "idle", &[0, 0]);
        });
        push_object_with_properties(bytes, "TransitionValueStringComparator", |bytes| {
            push_string_property(bytes, "TransitionValueStringComparator", "value", "ready");
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn color_fixture_bytes(file_id: u64) -> Vec<u8> {
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
            push_color_property(
                bytes,
                "ViewModelInstanceColor",
                "propertyValue",
                0xff00_aa44,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "alternate");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
            push_color_property(
                bytes,
                "ViewModelInstanceColor",
                "propertyValue",
                0xff00_bb55,
            );
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_state_machine(bytes, |bytes| {
            push_bindable_color_data_bind_context(bytes, 0xff00_0000, &[0, 0]);
        });
        push_object_with_properties(bytes, "TransitionValueColorComparator", |bytes| {
            push_color_property(
                bytes,
                "TransitionValueColorComparator",
                "value",
                0xff00_aa44,
            );
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

#[derive(Clone, Copy)]
enum FormulaContextSourceKind {
    Color,
    String,
}

#[derive(Clone, Copy)]
enum StringConverterKind {
    ToString,
    Trim,
    RemoveZeros,
    Pad,
}

#[derive(Clone, Copy)]
enum ToStringSourceKind {
    Number,
    Boolean,
    Trigger,
    SymbolListIndex,
    Color,
}

impl ToStringSourceKind {
    fn property_name(self) -> &'static str {
        match self {
            Self::Number => "amount",
            Self::Boolean => "enabled",
            Self::Trigger => "fire",
            Self::SymbolListIndex => "symbol",
            Self::Color => "tint",
        }
    }

    fn expected_value(self) -> &'static str {
        match self {
            Self::Number => "12,345.5",
            Self::Boolean => "1",
            Self::Trigger => "3",
            Self::SymbolListIndex => "7",
            Self::Color => "rgba(64,32,16,128)|#40201080",
        }
    }
}

impl StringConverterKind {
    fn source_value(self) -> &'static str {
        match self {
            Self::ToString => "ready",
            Self::Trim => "  ready\t",
            Self::RemoveZeros => "120.3400",
            Self::Pad => "go",
        }
    }

    fn expected_value(self) -> &'static str {
        match self {
            Self::ToString | Self::Trim => "ready",
            Self::RemoveZeros => "120.34",
            Self::Pad => "goxyx",
        }
    }
}

fn push_bindable_number_data_bind_context_with_converter(
    bytes: &mut Vec<u8>,
    value: f32,
    path: &[u32],
    converter_id: u64,
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
    });
}

fn string_converter_condition_fixture_bytes(
    file_id: u64,
    converter_kind: StringConverterKind,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
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
                converter_kind.source_value(),
            );
        });
        match converter_kind {
            StringConverterKind::ToString => {
                push_object_with_properties(bytes, "DataConverterToString", |_| {});
            }
            StringConverterKind::Trim => {
                push_object_with_properties(bytes, "DataConverterStringTrim", |bytes| {
                    push_uint_property(bytes, "DataConverterStringTrim", "trimType", 3);
                });
            }
            StringConverterKind::RemoveZeros => {
                push_object_with_properties(bytes, "DataConverterStringRemoveZeros", |_| {});
            }
            StringConverterKind::Pad => {
                push_object_with_properties(bytes, "DataConverterStringPad", |bytes| {
                    push_uint_property(bytes, "DataConverterStringPad", "length", 5);
                    push_uint_property(bytes, "DataConverterStringPad", "padType", 1);
                    push_string_property(bytes, "DataConverterStringPad", "text", "xy");
                });
            }
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
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_bindable_string_data_bind_context_with_converter_and_flags(
            bytes,
            "idle",
            &[0, 0],
            Some(0),
            0,
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
                converter_kind.expected_value(),
            );
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn to_string_converter_condition_fixture_bytes(
    file_id: u64,
    source_kind: ToStringSourceKind,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        match source_kind {
            ToStringSourceKind::Number => {
                push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
                });
            }
            ToStringSourceKind::Boolean => {
                push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
                });
            }
            ToStringSourceKind::Trigger => {
                push_object_with_properties(bytes, "ViewModelPropertyTrigger", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyTrigger", "name", "fire");
                });
            }
            ToStringSourceKind::SymbolListIndex => {
                push_object_with_properties(bytes, "ViewModelPropertySymbolListIndex", |bytes| {
                    push_string_property(
                        bytes,
                        "ViewModelPropertySymbolListIndex",
                        "name",
                        "symbol",
                    );
                });
            }
            ToStringSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
                });
            }
        }
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        match source_kind {
            ToStringSourceKind::Number => {
                push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
                    push_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 12345.5);
                });
            }
            ToStringSourceKind::Boolean => {
                push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
                    push_bool_property(bytes, "ViewModelInstanceBoolean", "propertyValue", true);
                });
            }
            ToStringSourceKind::Trigger => {
                push_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 0);
                    push_uint_property(bytes, "ViewModelInstanceTrigger", "propertyValue", 3);
                });
            }
            ToStringSourceKind::SymbolListIndex => {
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
                        7,
                    );
                });
            }
            ToStringSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
                    push_color_property(
                        bytes,
                        "ViewModelInstanceColor",
                        "propertyValue",
                        0x8040_2010,
                    );
                });
            }
        }
        push_object_with_properties(bytes, "DataConverterToString", |bytes| match source_kind {
            ToStringSourceKind::Number => {
                push_uint_property(bytes, "DataConverterToString", "flags", 1 | 2 | 4);
                push_uint_property(bytes, "DataConverterToString", "decimals", 2);
            }
            ToStringSourceKind::Color => {
                push_string_property(
                    bytes,
                    "DataConverterToString",
                    "colorFormat",
                    "rgba(%r,%g,%b,%a)|#%R%G%B%A",
                );
            }
            ToStringSourceKind::Boolean
            | ToStringSourceKind::Trigger
            | ToStringSourceKind::SymbolListIndex => {}
        });
        push_string_converter_transition_tail(bytes, 0, source_kind.expected_value());
    })
}

fn string_converter_group_condition_fixture_bytes(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "  go  ");
        });
        push_object_with_properties(bytes, "DataConverterStringTrim", |bytes| {
            push_uint_property(bytes, "DataConverterStringTrim", "trimType", 3);
        });
        push_object_with_properties(bytes, "DataConverterStringPad", |bytes| {
            push_uint_property(bytes, "DataConverterStringPad", "length", 5);
            push_uint_property(bytes, "DataConverterStringPad", "padType", 1);
            push_string_property(bytes, "DataConverterStringPad", "text", "xy");
        });
        push_object_with_properties(bytes, "DataConverterGroup", |_| {});
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_string_converter_transition_tail(bytes, 2, "goxyx");
    })
}

fn push_string_converter_transition_tail(
    bytes: &mut Vec<u8>,
    converter_id: u64,
    expected_value: &str,
) {
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
        "idle",
        &[0, 0],
        Some(converter_id),
        0,
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
            expected_value,
        );
    });
    push_object_with_properties(bytes, "AnimationState", |bytes| {
        push_uint_property(bytes, "AnimationState", "animationId", 1);
    });
    push_object_with_properties(bytes, "ExitState", |_| {});
}

fn formula_context_fixture_bytes(file_id: u64, source_kind: FormulaContextSourceKind) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyString", "name", "amount");
                });
            }
        }
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
                    push_color_property(bytes, "ViewModelInstanceColor", "propertyValue", 1);
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
                    push_string_property(
                        bytes,
                        "ViewModelInstanceString",
                        "propertyValue",
                        "1.0suffix",
                    );
                });
            }
        }
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
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_bindable_color_data_bind_context(bytes, 0xff00_0000, &[0, 0]);
            }
            FormulaContextSourceKind::String => {
                push_bindable_string_data_bind_context(bytes, "idle", &[0, 0]);
            }
        }
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn formula_reverse_flow_fixture_bytes(
    file_id: u64,
    source_kind: FormulaContextSourceKind,
    data_bind_flags: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyString", "name", "amount");
                });
            }
        }
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
                    push_color_property(bytes, "ViewModelInstanceColor", "propertyValue", 1);
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
                    push_string_property(
                        bytes,
                        "ViewModelInstanceString",
                        "propertyValue",
                        "1.0suffix",
                    );
                });
            }
        }
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
        let mut source_path_ids = Vec::new();
        for path_id in [0_u32, 0] {
            push_var_uint(&mut source_path_ids, u64::from(path_id));
        }
        push_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
            push_f32_property(bytes, "BindablePropertyNumber", "propertyValue", 0.75);
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
            push_uint_property(bytes, "DataBindContext", "converterId", 0);
            push_uint_property(bytes, "DataBindContext", "flags", data_bind_flags);
        });
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn random_formula_source_change_fixture_bytes(
    file_id: u64,
    source_kind: FormulaContextSourceKind,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
                    push_string_property(bytes, "ViewModelPropertyString", "name", "amount");
                });
            }
        }
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        match source_kind {
            FormulaContextSourceKind::Color => {
                push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
                    push_color_property(bytes, "ViewModelInstanceColor", "propertyValue", 1);
                });
            }
            FormulaContextSourceKind::String => {
                push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
                    push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
                    push_string_property(
                        bytes,
                        "ViewModelInstanceString",
                        "propertyValue",
                        "1.0suffix",
                    );
                });
            }
        }
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_f32_property(bytes, "DataConverterOperationValue", "operationValue", 2.0);
        });
        push_object_with_properties(bytes, "DataConverterFormula", |bytes| {
            push_uint_property(bytes, "DataConverterFormula", "randomModeValue", 2);
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
        push_bindable_number_data_bind_context_with_converter(bytes, 0.75, &[0, 0], 2);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn nested_string_fixture_bytes(file_id: u64, imported_child_value: &str) -> Vec<u8> {
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
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(
                bytes,
                "ViewModelInstanceString",
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
        push_bindable_string_data_bind_context(bytes, "idle", &[0, 0, 0]);
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueStringComparator", |bytes| {
            push_string_property(bytes, "TransitionValueStringComparator", "value", "ready");
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn nested_color_fixture_bytes(file_id: u64, imported_child_value: u32) -> Vec<u8> {
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
        push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
            push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 0);
            push_color_property(
                bytes,
                "ViewModelInstanceColor",
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
        push_bindable_color_data_bind_context(bytes, 0xff00_0000, &[0, 0, 0]);
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueColorComparator", |bytes| {
            push_color_property(
                bytes,
                "TransitionValueColorComparator",
                "value",
                0xff00_aa44,
            );
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn string_public_update_fixture_bytes(file_id: u64) -> Vec<u8> {
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "ready");
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
            "idle",
            &[0, 0],
            Some(0),
            DATA_BIND_TWO_WAY,
        );
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueStringComparator", |bytes| {
            push_string_property(bytes, "TransitionValueStringComparator", "value", "ready");
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn color_public_update_fixture_bytes(file_id: u64) -> Vec<u8> {
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;

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
            push_color_property(
                bytes,
                "ViewModelInstanceColor",
                "propertyValue",
                0xff00_aa44,
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
        push_bindable_color_data_bind_context_with_flags(
            bytes,
            0xff00_0000,
            &[0, 0],
            DATA_BIND_TWO_WAY,
        );
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 0);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueColorComparator", |bytes| {
            push_color_property(
                bytes,
                "TransitionValueColorComparator",
                "value",
                0xff00_aa44,
            );
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn compare_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    label: &str,
) {
    compare_advance_impl(cpp, rust, advanced, false, label);
}

fn compare_advance_impl(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    advanced: bool,
    allow_view_model_triggers: bool,
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
    if !allow_view_model_triggers {
        assert!(cpp.view_model_triggers.is_empty());
    }
}

fn authored_data_bind(
    machine: &StateMachineInstance,
    data_bind_index: usize,
    label: &str,
) -> nuxie_runtime::CoreHandle {
    machine
        .state_machine()
        .with_downcast::<StateMachine, _>(|machine| machine.data_bind(data_bind_index))
        .flatten()
        .unwrap_or_else(|| panic!("missing authored data bind {data_bind_index} for {label}"))
}

fn source_property_at(
    machine: &StateMachineInstance,
    data_bind_index: usize,
    label: &str,
) -> nuxie_runtime::CoreHandle {
    let authored_bind = authored_data_bind(machine, data_bind_index, label);
    let path = authored_bind
        .with_downcast::<DataBindContext, _>(|bind| bind.source_path_ids().to_vec())
        .unwrap_or_else(|| panic!("missing authored data-bind context for {label}"));
    machine
        .view_model_property(&path)
        .unwrap_or_else(|| panic!("missing native bind source for {label}"))
}

fn bindable_property_at(
    machine: &StateMachineInstance,
    data_bind_index: usize,
    label: &str,
) -> nuxie_runtime::CoreHandle {
    let authored_bind = authored_data_bind(machine, data_bind_index, label);
    let authored_property = authored_bind
        .with(|bind| bind.as_data_bind().and_then(|bind| bind.target()))
        .flatten()
        .unwrap_or_else(|| panic!("missing authored bindable property for {label}"));
    machine
        .bindable_property_instance(&authored_property)
        .unwrap_or_else(|| panic!("missing native bindable property occurrence for {label}"))
}

fn live_data_bind_at(
    machine: &StateMachineInstance,
    data_bind_index: usize,
    label: &str,
) -> nuxie_runtime::CoreHandle {
    let target = bindable_property_at(machine, data_bind_index, label);
    machine
        .bindable_data_bind_to_target(&target)
        .or_else(|| machine.bindable_data_bind_to_source(&target))
        .unwrap_or_else(|| panic!("missing native live data bind {data_bind_index} for {label}"))
}

fn source_property(machine: &StateMachineInstance, label: &str) -> nuxie_runtime::CoreHandle {
    source_property_at(machine, 0, label)
}

fn bindable_property(machine: &StateMachineInstance, label: &str) -> nuxie_runtime::CoreHandle {
    bindable_property_at(machine, 0, label)
}

fn compare_string_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    let binding = cpp
        .string_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .unwrap_or_else(|| panic!("missing C++ string binding for {label}"));
    rust.with_instance(|rust| {
        let source = source_property(rust, label)
            .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value);
        let target =
            bindable_property(rust, label).with_downcast::<BindablePropertyString, _>(|property| {
                property.base.property_value().to_owned()
            });
        assert_eq!(binding.source_value.as_deref(), source.as_deref());
        assert_eq!(binding.target_value.as_deref(), target.as_deref());
    });
}

fn compare_string_converter_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    converter_kind: StringConverterKind,
    label: &str,
) {
    compare_string_binding(cpp, rust, label);
    rust.with_instance(|rust| {
        let converter = live_data_bind_at(rust, 0, label)
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.converter()))
            .flatten()
            .unwrap_or_else(|| panic!("missing native string converter for {label}"));
        let correct_type = match converter_kind {
            StringConverterKind::ToString => converter
                .with_downcast::<DataConverterToString, _>(|_| true)
                .unwrap_or(false),
            StringConverterKind::Trim => converter
                .with_downcast::<DataConverterStringTrim, _>(|_| true)
                .unwrap_or(false),
            StringConverterKind::RemoveZeros => converter
                .with_downcast::<DataConverterStringRemoveZeros, _>(|_| true)
                .unwrap_or(false),
            StringConverterKind::Pad => converter
                .with_downcast::<DataConverterStringPad, _>(|_| true)
                .unwrap_or(false),
        };
        assert!(
            correct_type,
            "wrong native string converter type for {label}"
        );
    });
}

fn compare_to_string_converter_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    source_kind: ToStringSourceKind,
    label: &str,
) {
    let binding = cpp
        .string_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .unwrap_or_else(|| panic!("missing C++ string binding for {label}"));
    rust.with_instance(|rust| {
        let source = source_property(rust, label);
        let source_matches = match source_kind {
            ToStringSourceKind::Number => source
                .with_downcast::<ViewModelInstanceNumber, _>(|property| property.value() == 12345.5)
                .unwrap_or(false),
            ToStringSourceKind::Boolean => source
                .with_downcast::<ViewModelInstanceBoolean, _>(ViewModelInstanceBoolean::value)
                .unwrap_or(false),
            ToStringSourceKind::Trigger => source
                .with_downcast::<ViewModelInstanceTrigger, _>(|property| {
                    property.base.property_value() == 3
                })
                .unwrap_or(false),
            ToStringSourceKind::SymbolListIndex => source
                .with_downcast::<ViewModelInstanceSymbolListIndex, _>(|property| {
                    property.base.property_value() == 7
                })
                .unwrap_or(false),
            ToStringSourceKind::Color => source
                .with_downcast::<ViewModelInstanceColor, _>(|property| {
                    property.value() as u32 == 0x8040_2010
                })
                .unwrap_or(false),
        };
        assert!(source_matches, "wrong native ToString source for {label}");

        let target =
            bindable_property(rust, label).with_downcast::<BindablePropertyString, _>(|property| {
                property.base.property_value().to_owned()
            });
        assert_eq!(binding.source_value, None, "{label} C++ string source");
        assert_eq!(binding.target_value.as_deref(), target.as_deref());

        let converter = live_data_bind_at(rust, 0, label)
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.converter()))
            .flatten()
            .unwrap_or_else(|| panic!("missing native ToString converter for {label}"));
        assert!(
            converter
                .with_downcast::<DataConverterToString, _>(|_| true)
                .unwrap_or(false),
            "wrong native ToString converter type for {label}"
        );
    });
}

fn compare_string_converter_group_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    compare_string_binding(cpp, rust, label);
    rust.with_instance(|rust| {
        let converter = live_data_bind_at(rust, 0, label)
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.converter()))
            .flatten()
            .unwrap_or_else(|| panic!("missing native string converter group for {label}"));
        let item_converters = converter
            .with_downcast::<DataConverterGroup, _>(|group| {
                group
                    .items()
                    .iter()
                    .map(|item| {
                        item.with(|item| {
                            item.as_data_converter_group_item()
                                .and_then(|item| item.converter())
                        })
                        .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(item_converters.len(), 2, "{label} native converter items");
        assert!(
            item_converters[0]
                .as_ref()
                .and_then(|converter| {
                    converter.with_downcast::<DataConverterStringTrim, _>(|_| true)
                })
                .unwrap_or(false),
            "{label} first converter is not StringTrim"
        );
        assert!(
            item_converters[1]
                .as_ref()
                .and_then(|converter| {
                    converter.with_downcast::<DataConverterStringPad, _>(|_| true)
                })
                .unwrap_or(false),
            "{label} second converter is not StringPad"
        );
    });
}

fn compare_color_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    let binding = cpp
        .color_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .unwrap_or_else(|| panic!("missing C++ color binding for {label}"));
    rust.with_instance(|rust| {
        let source = source_property(rust, label)
            .with_downcast::<ViewModelInstanceColor, _>(|property| property.value() as u32);
        let target =
            bindable_property(rust, label).with_downcast::<BindablePropertyColor, _>(|property| {
                property.base.property_value() as u32
            });
        assert_eq!(binding.source_value, source);
        assert_eq!(binding.target_value, target);
    });
}

fn compare_formula_number_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    let binding = cpp
        .number_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .unwrap_or_else(|| panic!("missing C++ number binding 0 for {label}"));
    rust.with_instance(|rust| {
        let live_bind = live_data_bind_at(rust, 0, label);
        let source_is_number = live_bind
            .with(|bind| {
                bind.as_data_bind()
                    .and_then(|bind| bind.source())
                    .is_some_and(|source| {
                        source
                            .with(|source| source.as_view_model_instance_number().is_some())
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false);
        let formula_is_native = live_bind
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.converter()))
            .flatten()
            .and_then(|converter| converter.with_downcast::<DataConverterFormula, _>(|_| true))
            .unwrap_or(false);
        let target = bindable_property_at(rust, 0, label)
            .with_downcast::<BindablePropertyNumber, _>(|property| property.base.property_value());
        assert!(
            !source_is_number,
            "{label} formula source must remain its native VMI type"
        );
        assert!(
            formula_is_native,
            "{label} missing native DataConverterFormula occurrence"
        );
        assert_eq!(binding.source_value, None, "{label} number sourceValue");
        match (binding.target_value, target) {
            (Some(cpp), Some(rust)) => assert_close(rust, cpp, label),
            (None, None) => {}
            (cpp, rust) => {
                panic!("{label} number targetValue presence mismatch: C++ {cpp:?}, Rust {rust:?}")
            }
        }
    });
}

fn compare_random_group_number_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    let binding = cpp
        .number_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 0)
        .unwrap_or_else(|| panic!("missing C++ number binding 0 for {label}"));
    rust.with_instance(|rust| {
        let live_bind = live_data_bind_at(rust, 0, label);
        let formula_is_native = live_bind
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.converter()))
            .flatten()
            .and_then(|converter| {
                converter
                    .with_downcast::<DataConverterGroup, _>(|group| {
                        group.items().get(1).and_then(|item| {
                            item.with(|item| {
                                item.as_data_converter_group_item()
                                    .and_then(|item| item.converter())
                            })
                            .flatten()
                        })
                    })
                    .flatten()
            })
            .and_then(|formula| formula.with_downcast::<DataConverterFormula, _>(|_| true))
            .unwrap_or(false);
        let target = bindable_property_at(rust, 0, label)
            .with_downcast::<BindablePropertyNumber, _>(|property| property.base.property_value());
        assert!(
            formula_is_native,
            "{label} missing native DataConverterFormula in group occurrence"
        );
        assert_eq!(binding.source_value, None, "{label} number sourceValue");
        match (binding.target_value, target) {
            (Some(cpp), Some(rust)) => assert_close(rust, cpp, label),
            (None, None) => {}
            (cpp, rust) => {
                panic!("{label} number targetValue presence mismatch: C++ {cpp:?}, Rust {rust:?}")
            }
        }
    });
}

fn compare_formula_string_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    compare_formula_number_binding(cpp, rust, label);
    let binding = cpp
        .string_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 1)
        .unwrap_or_else(|| panic!("missing C++ string binding 1 for {label}"));
    rust.with_instance(|rust| {
        let source = source_property_at(rust, 1, label)
            .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value);
        let target = bindable_property_at(rust, 1, label)
            .with_downcast::<BindablePropertyString, _>(|property| {
                property.base.property_value().to_owned()
            });
        assert_eq!(binding.source_value.as_deref(), source.as_deref());
        assert_eq!(binding.target_value.as_deref(), target.as_deref());
    });
}

fn compare_formula_color_binding(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &RuntimeStateMachineInstanceHandle,
    label: &str,
) {
    compare_formula_number_binding(cpp, rust, label);
    let binding = cpp
        .color_bindings
        .iter()
        .find(|binding| binding.data_bind_index == 1)
        .unwrap_or_else(|| panic!("missing C++ color binding 1 for {label}"));
    rust.with_instance(|rust| {
        let source = source_property_at(rust, 1, label)
            .with_downcast::<ViewModelInstanceColor, _>(|property| property.value() as u32);
        let target = bindable_property_at(rust, 1, label)
            .with_downcast::<BindablePropertyColor, _>(|property| {
                property.base.property_value() as u32
            });
        assert_eq!(binding.source_value, source);
        assert_eq!(binding.target_value, target);
    });
}

fn compare_bound_node_x(cpp: &CppArtboard, rust: &NativeFixture, label: &str) {
    NativeArtboard::update_components_handle(&rust.artboard.core_handle());
    let cpp_node =
        cpp.runtime_update
            .as_ref()
            .and_then(|update| {
                update.components.iter().find(|component| {
                    component.local_id != 0 && component.local_transform.is_some()
                })
            })
            .expect("C++ bound transform component");
    let cpp_x = cpp_node.local_transform.expect("C++ bound transform")[4];
    let rust_x = rust
        .artboard
        .with_artboard(|artboard| artboard.object_handle_at::<Node>(1))
        .and_then(|node| node.with_downcast::<Node, _>(|node| node.x()))
        .expect("native bound transform x");
    assert_close(rust_x, cpp_x, label);
}

fn run_default_bind_source(
    label: &str,
    bytes: Vec<u8>,
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    run_default_bind_source_impl(label, bytes, false, compare_binding);
}

fn run_default_bind_source_impl(
    label: &str,
    bytes: Vec<u8>,
    allow_view_model_triggers: bool,
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
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
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    let first_advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance_impl(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        first_advanced,
        allow_view_model_triggers,
        label,
    );

    rust.bind_default_view_model(label);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance_impl(
            cpp_advance,
            &rust.machine,
            advanced,
            allow_view_model_triggers,
            label,
        );
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_string_converter_case(label: &str, file_id: u64, converter_kind: StringConverterKind) {
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
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let bytes = string_converter_condition_fixture_bytes(file_id, converter_kind);
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let rust = native_fixture(&bytes, label);
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

    rust.bind_default_view_model(label);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_string_converter_binding(cpp_advance, &rust.machine, converter_kind, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_context_bind_source(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    bind_context: impl FnOnce(&NativeFixture, &str) -> nuxie_runtime::CoreHandle,
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
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

    let _active_context = bind_context(&rust, label);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_formula_context(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    bind_context: impl FnOnce(&NativeFixture, &str) -> nuxie_runtime::CoreHandle,
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 2);

    let _active_context = bind_context(&rust, label);
    for (cpp_advance, seconds) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_formula_source_mutation(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    bind_context: impl FnOnce(&NativeFixture, &str) -> nuxie_runtime::CoreHandle,
    mutate_source: impl Fn(&nuxie_runtime::CoreHandle, &str),
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    let active_context = bind_context(&rust, label);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    compare_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        label,
    );

    mutate_source(&active_context, label);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_random_formula_source_change(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    mutate_source: impl Fn(&NativeFixture, &nuxie_runtime::CoreHandle, &str),
    assert_source: impl Fn(&NativeFixture, &nuxie_runtime::CoreHandle, usize, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let seeded_random_values = [0.25_f32, 0.75, 0.5];
    let probe_args = counted_runtime_random_probe_args(&seeded_random_values, args);
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &probe_args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 3);

    let _random_lock = RANDOM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let source = rust.bind_default_view_model(label);
    let _random_values = set_runtime_random_test_values(&seeded_random_values);
    assert_source(&rust, &source, 0, label);

    let expected_counts = [1_usize, 2, 2];
    for (report_index, (cpp_advance, seconds)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip([0.0, 0.0, 1.0])
        .enumerate()
    {
        if report_index == 1 {
            mutate_source(&rust, &source, label);
        }
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_random_group_number_binding(cpp_advance, &rust.machine, label);
        assert_source(&rust, &source, report_index, label);
        assert_eq!(
            cpp_advance.random_total_calls, expected_counts[report_index],
            "{label} C++ random totalCalls at report {report_index}"
        );
        assert_eq!(
            RandomProvider::total_calls() as usize,
            expected_counts[report_index],
            "{label} native random totalCalls at report {report_index}"
        );
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_public_update_target_to_source(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    set_target: impl Fn(&NativeFixture, &str),
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    rust.bind_default_view_model(label);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    compare_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        label,
    );

    set_target(&rust, label);
    DataBindContainerOwner::StateMachine(rust.machine.downgrade()).update_data_binds(true);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    compare_binding(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
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
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_formula_explicit_target_to_source(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    assert_source: impl Fn(&NativeFixture, &nuxie_runtime::CoreHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    let source = rust.bind_default_view_model(label);
    rust.machine
        .with_instance_mut(StateMachineInstance::advanced_data_context);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        false,
        label,
    );
    compare_formula_number_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        label,
    );
    assert_source(&rust, &source, label);

    rust.set_bindable_number(0.4, label);
    rust.machine
        .with_instance_mut(StateMachineInstance::advanced_data_context);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    compare_formula_number_binding(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        label,
    );
    assert_source(&rust, &source, label);

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_formula_number_binding(cpp_advance, &rust.machine, label);
        assert_source(&rust, &source, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_formula_public_update_target_to_source(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    assert_source: impl Fn(&NativeFixture, &nuxie_runtime::CoreHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
    let cpp_artboard = cpp.artboards.first().expect("C++ artboard");
    assert_eq!(cpp_artboard.runtime_state_machine_advances.len(), 4);

    let source = rust.bind_default_view_model(label);
    let advanced = rust
        .machine
        .with_instance_mut(|machine| machine.advance_seconds(0.0));
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        advanced,
        label,
    );
    compare_formula_number_binding(
        &cpp_artboard.runtime_state_machine_advances[0],
        &rust.machine,
        label,
    );
    assert_source(&rust, &source, label);

    rust.set_bindable_number(0.4, label);
    DataBindContainerOwner::StateMachine(rust.machine.downgrade()).update_data_binds(true);
    compare_advance(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        false,
        label,
    );
    compare_formula_number_binding(
        &cpp_artboard.runtime_state_machine_advances[1],
        &rust.machine,
        label,
    );
    assert_source(&rust, &source, label);

    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[2..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_formula_number_binding(cpp_advance, &rust.machine, label);
        assert_source(&rust, &source, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

fn run_default_source_handle_mutation(
    label: &str,
    bytes: Vec<u8>,
    args: &[String],
    mutate_source: impl FnOnce(&NativeFixture, &nuxie_runtime::CoreHandle, &str),
    compare_binding: impl Fn(&CppRuntimeStateMachineAdvance, &RuntimeStateMachineInstanceHandle, &str),
) {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, args);
    let rust = native_fixture(&bytes, label);
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

    let source_instance = rust.bind_default_view_model(label);
    mutate_source(&rust, &source_instance, label);
    for (cpp_advance, seconds) in cpp_artboard.runtime_state_machine_advances[1..]
        .iter()
        .zip([0.0, 1.0])
    {
        let advanced = rust
            .machine
            .with_instance_mut(|machine| machine.advance_seconds(seconds));
        compare_advance(cpp_advance, &rust.machine, advanced, label);
        compare_binding(cpp_advance, &rust.machine, label);
    }
    compare_bound_node_x(cpp_artboard, &rust, label);
}

#[test]
fn state_machine_default_viewmodel_string_bind_source_matches_cpp_probe() {
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_string_bind_cpp.riv",
        string_fixture_bytes(8372),
        compare_string_binding,
    );
}

#[test]
fn state_machine_default_viewmodel_color_bind_source_matches_cpp_probe() {
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_color_bind_cpp.riv",
        color_fixture_bytes(8373),
        compare_color_binding,
    );
}

#[test]
fn state_machine_external_viewmodel_string_bind_source_matches_cpp_probe() {
    let value = "ready";
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-string".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_owned(),
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_external_viewmodel_string_bind_cpp.riv",
        string_fixture_bytes(8389),
        &args,
        |rust, label| {
            rust.set_bindable_string(value, label);
            rust.bind_external_view_model(1, label)
        },
        compare_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_string_bind_source_matches_cpp_probe() {
    let value = "ready";
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-string-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_string_bind_cpp.riv",
        string_fixture_bytes(8397),
        &args,
        |rust, label| rust.bind_owned_string(value, label),
        compare_string_binding,
    );
}

#[test]
fn state_machine_external_viewmodel_color_bind_source_matches_cpp_probe() {
    let value = 0xff00_aa44_u32;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-color".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_string(),
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_external_viewmodel_color_bind_cpp.riv",
        color_fixture_bytes(8390),
        &args,
        |rust, label| {
            rust.set_bindable_color(value, label);
            rust.bind_external_view_model(1, label)
        },
        compare_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_color_bind_source_matches_cpp_probe() {
    let value = 0xff00_aa44_u32;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-color-state-machine-context".to_owned(),
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_color_bind_cpp.riv",
        color_fixture_bytes(8398),
        &args,
        |rust, label| rust.bind_owned_color(value, label),
        compare_color_binding,
    );
}

#[test]
fn string_to_string_public_update_target_to_source_matches_cpp_probe() {
    let edit = "manual";
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-string".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        edit.to_owned(),
        "--runtime-update-state-machine-data-binds".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_public_update_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_string_to_string_public_update_target_to_source_cpp.riv",
        string_public_update_fixture_bytes(8544),
        &args,
        |rust, label| rust.set_bindable_string(edit, label),
        compare_string_binding,
    );
}

#[test]
fn color_public_update_target_to_source_matches_cpp_probe() {
    let forced_value = 0xff33_7766_u32;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-color".to_owned(),
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
    run_public_update_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_color_public_update_target_to_source_cpp.riv",
        color_public_update_fixture_bytes(8653),
        &args,
        |rust, label| rust.set_bindable_color(forced_value, label),
        compare_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_string_source_handle_bind_source_matches_cpp_probe() {
    let value = "ready";
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-string-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_string_source_handle_bind_cpp.riv",
        string_fixture_bytes(8764),
        &args,
        |rust, label| rust.bind_owned_string(value, label),
        compare_string_binding,
    );
}

#[test]
fn state_machine_default_viewmodel_string_source_handle_mutation_matches_cpp_probe() {
    let property_name = "label";
    let value = "handled";
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-string-by-name".to_owned(),
        "0".to_owned(),
        property_name.to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_default_source_handle_mutation(
        "synthetic/runtime_state_machine_default_viewmodel_string_source_handle_mutation_cpp.riv",
        string_fixture_bytes(8743),
        &args,
        |rust, source, label| rust.set_default_string_property(source, property_name, value, label),
        compare_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_color_source_handle_bind_source_matches_cpp_probe() {
    let value = 0xff00_aa44_u32;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-color-state-machine-context".to_owned(),
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_color_source_handle_bind_cpp.riv",
        color_fixture_bytes(8765),
        &args,
        |rust, label| rust.bind_owned_color(value, label),
        compare_color_binding,
    );
}

#[test]
fn state_machine_default_viewmodel_color_source_handle_mutation_matches_cpp_probe() {
    let property_name = "tint";
    let value = 0x6f11_4455_u32;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-color-by-name".to_owned(),
        "0".to_owned(),
        property_name.to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_default_source_handle_mutation(
        "synthetic/runtime_state_machine_default_viewmodel_color_source_handle_mutation_cpp.riv",
        color_fixture_bytes(8744),
        &args,
        |rust, source, label| rust.set_default_color_property(source, property_name, value, label),
        compare_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_nested_string_name_path_bind_source_matches_cpp_probe() {
    let value = "ready";
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-string-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/label".to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_nested_string_name_path_bind_cpp.riv",
        nested_string_fixture_bytes(8580, "idle"),
        &args,
        |rust, label| rust.bind_owned_nested_string(value, label),
        compare_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_nested_color_name_path_bind_source_matches_cpp_probe() {
    let value = 0xff00_aa44_u32;
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-bind-owned-view-model-color-name-path-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "child/tint".to_owned(),
        value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_nested_color_name_path_bind_cpp.riv",
        nested_color_fixture_bytes(8581, 0xff00_0000),
        &args,
        |rust, label| rust.bind_owned_nested_color(value, label),
        compare_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_imported_intermediate_string_source_matches_cpp_probe() {
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_imported_intermediate_string_source_cpp.riv",
        nested_string_fixture_bytes(8590, "ready"),
        &args,
        NativeFixture::bind_owned_imported_child,
        compare_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_imported_intermediate_color_source_matches_cpp_probe() {
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
    run_context_bind_source(
        "synthetic/runtime_state_machine_owned_viewmodel_imported_intermediate_color_source_cpp.riv",
        nested_color_fixture_bytes(8591, 0xff00_aa44),
        &args,
        NativeFixture::bind_owned_imported_child,
        compare_color_binding,
    );
}

#[test]
fn state_machine_imported_viewmodel_color_formula_context_matches_cpp_probe() {
    let value = 0x7f11_2233_u32;
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-view-model-instance-source-color".to_owned(),
        "0".to_owned(),
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
    run_formula_context(
        "synthetic/runtime_state_machine_imported_viewmodel_color_formula_context_cpp.riv",
        formula_context_fixture_bytes(8970, FormulaContextSourceKind::Color),
        &args,
        |rust, label| {
            let imported = rust.bind_external_view_model(0, label);
            rust.set_default_color_property(&imported, "tint", value, label);
            imported
        },
        compare_formula_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_color_formula_context_matches_cpp_probe() {
    let value = 0xff00_aa44_u32;
    let args = [
        "--runtime-bind-owned-view-model-color-state-machine-context".to_owned(),
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
    run_formula_context(
        "synthetic/runtime_state_machine_owned_viewmodel_color_formula_context_cpp.riv",
        formula_context_fixture_bytes(8971, FormulaContextSourceKind::Color),
        &args,
        |rust, label| {
            let owned = rust.create_owned_view_model(label);
            set_view_model_color_property(&owned, "tint", value, label);
            rust.machine
                .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
            owned
        },
        compare_formula_color_binding,
    );
}

#[test]
fn state_machine_imported_viewmodel_string_formula_context_matches_cpp_probe() {
    let value = "2.0suffix";
    let args = [
        "--runtime-bind-view-model-instance-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-view-model-instance-source-string".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_context(
        "synthetic/runtime_state_machine_imported_viewmodel_string_formula_context_cpp.riv",
        formula_context_fixture_bytes(8990, FormulaContextSourceKind::String),
        &args,
        |rust, label| {
            let imported = rust.bind_external_view_model(0, label);
            rust.set_default_string_property(&imported, "amount", value, label);
            imported
        },
        compare_formula_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_string_formula_context_matches_cpp_probe() {
    let value = "2.0suffix";
    let args = [
        "--runtime-bind-owned-view-model-string-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_context(
        "synthetic/runtime_state_machine_owned_viewmodel_string_formula_context_cpp.riv",
        formula_context_fixture_bytes(8991, FormulaContextSourceKind::String),
        &args,
        |rust, label| {
            let owned = rust.create_owned_view_model(label);
            set_view_model_string_property(&owned, "amount", value, label);
            rust.machine
                .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
            owned
        },
        compare_formula_string_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_color_formula_source_mutation_matches_cpp_probe() {
    let initial_value = 0xff00_aa44_u32;
    let mutated_value = 0x7f11_2233_u32;
    let args = [
        "--runtime-bind-owned-view-model-color-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        initial_value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-owned-view-model-source-color".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        mutated_value.to_string(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_source_mutation(
        "synthetic/runtime_state_machine_owned_viewmodel_color_formula_source_mutation_cpp.riv",
        formula_context_fixture_bytes(9311, FormulaContextSourceKind::Color),
        &args,
        |rust, label| {
            let owned = rust.create_owned_view_model(label);
            set_view_model_color_property(&owned, "tint", initial_value, label);
            rust.machine
                .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
            owned
        },
        |source, label| {
            set_view_model_color_property(source, "tint", mutated_value, label);
        },
        compare_formula_color_binding,
    );
}

#[test]
fn state_machine_owned_viewmodel_string_formula_source_mutation_matches_cpp_probe() {
    let initial_value = "2.0suffix";
    let mutated_value = "3.0suffix";
    let args = [
        "--runtime-bind-owned-view-model-string-state-machine-context".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        initial_value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-owned-view-model-source-string".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        mutated_value.to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_source_mutation(
        "synthetic/runtime_state_machine_owned_viewmodel_string_formula_source_mutation_cpp.riv",
        formula_context_fixture_bytes(9312, FormulaContextSourceKind::String),
        &args,
        |rust, label| {
            let owned = rust.create_owned_view_model(label);
            set_view_model_string_property(&owned, "amount", initial_value, label);
            rust.machine
                .with_instance_mut(|machine| machine.bind_view_model_instance(owned.clone()));
            owned
        },
        |source, label| {
            set_view_model_string_property(source, "amount", mutated_value, label);
        },
        compare_formula_string_binding,
    );
}

#[test]
fn state_machine_default_viewmodel_color_formula_random_function_group_source_change_matches_cpp_probe()
 {
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-color".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "2".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_random_formula_source_change(
        "synthetic/runtime_state_machine_default_viewmodel_color_formula_random_function_group_call_count_source_change_cpp.riv",
        random_formula_source_change_fixture_bytes(9168, FormulaContextSourceKind::Color),
        &args,
        |rust, source, label| rust.set_default_color_property(source, "tint", 2, label),
        |rust, source, report_index, label| {
            let expected = if report_index == 0 { 1 } else { 2 };
            let actual =
                rust.default_view_model_property(source, "tint", label)
                    .with_downcast::<ViewModelInstanceColor, _>(|property| property.value() as u32);
            assert_eq!(actual, Some(expected), "{label} native color VMI source");
        },
    );
}

#[test]
fn state_machine_default_viewmodel_string_formula_random_function_group_source_change_matches_cpp_probe()
 {
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-default-view-model-source-string".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "2.0suffix".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_random_formula_source_change(
        "synthetic/runtime_state_machine_default_viewmodel_string_formula_random_function_group_call_count_source_change_cpp.riv",
        random_formula_source_change_fixture_bytes(9171, FormulaContextSourceKind::String),
        &args,
        |rust, source, label| {
            rust.set_default_string_property(source, "amount", "2.0suffix", label);
        },
        |rust, source, report_index, label| {
            let expected = if report_index == 0 {
                "1.0suffix"
            } else {
                "2.0suffix"
            };
            let actual =
                rust.default_view_model_property(source, "amount", label)
                    .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value);
            assert_eq!(
                actual.as_deref(),
                Some(expected),
                "{label} native string VMI source"
            );
        },
    );
}

#[test]
fn state_machine_default_viewmodel_color_formula_fallback_explicit_target_to_source_matches_cpp_probe()
 {
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.4".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_explicit_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_color_formula_fallback_explicit_target_to_source_cpp.riv",
        formula_reverse_flow_fixture_bytes(
            8901,
            FormulaContextSourceKind::Color,
            DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY,
        ),
        &args,
        |rust, source, label| {
            let actual =
                rust.default_view_model_property(source, "tint", label)
                    .with_downcast::<ViewModelInstanceColor, _>(|property| property.value() as u32);
            assert_eq!(actual, Some(1), "{label} native color formula source");
        },
    );
}

#[test]
fn state_machine_default_viewmodel_string_formula_fallback_explicit_target_to_source_matches_cpp_probe()
 {
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.4".to_owned(),
        "--runtime-advance-state-machine-data-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_explicit_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_string_formula_fallback_explicit_target_to_source_cpp.riv",
        formula_reverse_flow_fixture_bytes(
            8902,
            FormulaContextSourceKind::String,
            DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY,
        ),
        &args,
        |rust, source, label| {
            let actual =
                rust.default_view_model_property(source, "amount", label)
                    .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value);
            assert_eq!(
                actual.as_deref(),
                Some("1.0suffix"),
                "{label} native string formula source"
            );
        },
    );
}

#[test]
fn state_machine_default_viewmodel_color_formula_fallback_public_update_target_to_source_matches_cpp_probe()
 {
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.4".to_owned(),
        "--runtime-update-state-machine-data-binds".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_public_update_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_color_formula_fallback_public_update_target_to_source_cpp.riv",
        formula_reverse_flow_fixture_bytes(
            8629,
            FormulaContextSourceKind::Color,
            DATA_BIND_TWO_WAY,
        ),
        &args,
        |rust, source, label| {
            let actual =
                rust.default_view_model_property(source, "tint", label)
                    .with_downcast::<ViewModelInstanceColor, _>(|property| property.value() as u32);
            assert_eq!(actual, Some(1), "{label} native color formula source");
        },
    );
}

#[test]
fn state_machine_default_viewmodel_string_formula_fallback_public_update_target_to_source_matches_cpp_probe()
 {
    const DATA_BIND_TWO_WAY: u64 = 1 << 1;
    let args = [
        "--runtime-bind-default-view-model-state-machine-context".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bindable-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.4".to_owned(),
        "--runtime-update-state-machine-data-binds".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    run_formula_public_update_target_to_source(
        "synthetic/runtime_state_machine_default_viewmodel_string_formula_fallback_public_update_target_to_source_cpp.riv",
        formula_reverse_flow_fixture_bytes(
            8630,
            FormulaContextSourceKind::String,
            DATA_BIND_TWO_WAY,
        ),
        &args,
        |rust, source, label| {
            let actual =
                rust.default_view_model_property(source, "amount", label)
                    .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value);
            assert_eq!(
                actual.as_deref(),
                Some("1.0suffix"),
                "{label} native string formula source"
            );
        },
    );
}

#[test]
fn state_machine_default_viewmodel_string_to_string_converter_matches_cpp_probe() {
    run_string_converter_case(
        "synthetic/runtime_state_machine_default_viewmodel_string_to_string_converter_cpp.riv",
        8414,
        StringConverterKind::ToString,
    );
}

#[test]
fn state_machine_default_viewmodel_string_trim_converter_matches_cpp_probe() {
    run_string_converter_case(
        "synthetic/runtime_state_machine_default_viewmodel_string_trim_converter_cpp.riv",
        8419,
        StringConverterKind::Trim,
    );
}

#[test]
fn state_machine_default_viewmodel_string_remove_zeros_converter_matches_cpp_probe() {
    run_string_converter_case(
        "synthetic/runtime_state_machine_default_viewmodel_string_remove_zeros_converter_cpp.riv",
        8420,
        StringConverterKind::RemoveZeros,
    );
}

#[test]
fn state_machine_default_viewmodel_string_pad_converter_matches_cpp_probe() {
    run_string_converter_case(
        "synthetic/runtime_state_machine_default_viewmodel_string_pad_converter_cpp.riv",
        8421,
        StringConverterKind::Pad,
    );
}

#[test]
fn state_machine_default_viewmodel_number_to_string_converter_matches_cpp_probe() {
    let source_kind = ToStringSourceKind::Number;
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_number_to_string_converter_cpp.riv",
        to_string_converter_condition_fixture_bytes(8412, source_kind),
        move |cpp, rust, label| compare_to_string_converter_binding(cpp, rust, source_kind, label),
    );
}

#[test]
fn state_machine_default_viewmodel_boolean_to_string_converter_matches_cpp_probe() {
    let source_kind = ToStringSourceKind::Boolean;
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_boolean_to_string_converter_cpp.riv",
        to_string_converter_condition_fixture_bytes(8413, source_kind),
        move |cpp, rust, label| compare_to_string_converter_binding(cpp, rust, source_kind, label),
    );
}

#[test]
fn state_machine_default_viewmodel_trigger_to_string_converter_matches_cpp_probe() {
    let source_kind = ToStringSourceKind::Trigger;
    run_default_bind_source_impl(
        "synthetic/runtime_state_machine_default_viewmodel_trigger_to_string_converter_cpp.riv",
        to_string_converter_condition_fixture_bytes(8415, source_kind),
        true,
        move |cpp, rust, label| compare_to_string_converter_binding(cpp, rust, source_kind, label),
    );
}

#[test]
fn state_machine_default_viewmodel_symbol_list_index_to_string_converter_matches_cpp_probe() {
    let source_kind = ToStringSourceKind::SymbolListIndex;
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_symbol_list_index_to_string_converter_cpp.riv",
        to_string_converter_condition_fixture_bytes(8416, source_kind),
        move |cpp, rust, label| compare_to_string_converter_binding(cpp, rust, source_kind, label),
    );
}

#[test]
fn state_machine_default_viewmodel_color_to_string_converter_matches_cpp_probe() {
    let source_kind = ToStringSourceKind::Color;
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_color_to_string_converter_cpp.riv",
        to_string_converter_condition_fixture_bytes(8417, source_kind),
        move |cpp, rust, label| compare_to_string_converter_binding(cpp, rust, source_kind, label),
    );
}

#[test]
fn state_machine_default_viewmodel_string_converter_group_matches_cpp_probe() {
    run_default_bind_source(
        "synthetic/runtime_state_machine_default_viewmodel_string_converter_group_cpp.riv",
        string_converter_group_condition_fixture_bytes(8422),
        compare_string_converter_group_binding,
    );
}
