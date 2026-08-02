use crate::ArtboardInstance;
use crate::animation::{
    LinearAnimationInstance, RuntimeInterpolator, RuntimeKeyedPropertyTarget,
    RuntimeLinearAnimation, RuntimeLinearAnimationHandle,
};
use crate::components::TransformProperty;
use crate::data_bind_graph::RuntimeDataBindGraphConverterBuildCache;
use crate::properties::artboard_index_for_graph;
use crate::scripting::{ScriptError, ScriptListenerActionDefinition};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod animation_reset_factory;
mod bindables;
mod blend_state_direct_instance;
mod data_bind_template;
mod data_converter_binding;
mod event_report;
mod focus_action_clear;
mod focus_action_target;
mod focus_action_traversal;
mod focus_listener_group;
mod focused_input_dispatch;
mod gamepad_batch;
mod gamepad_listener_group;
mod instance;
mod keyboard_listener_group;
mod layer_state;
mod listener_action;
mod listener_action_owner;
mod listener_align_target;
mod listener_bool_change;
mod listener_fire_event;
mod listener_input_change;
mod listener_invocation;
mod listener_number_change;
mod listener_trigger_change;
mod listener_types;
mod listener_viewmodel_change;
mod nested_state_machine;
mod scripted_listener_action;
mod scripted_object_lifecycle;
mod scripted_transition_condition;
mod semantic_listener_group;
mod state_instance;
mod state_machine;
mod state_machine_fire_action;
mod state_machine_fire_event;
mod state_machine_fire_trigger;
mod state_machine_input;
mod state_machine_input_instance;
mod state_machine_instance;
mod state_machine_layer;
mod state_machine_layer_instance;
mod state_machine_listener;
mod state_machine_listener_single;
mod state_transition;
mod system_state_instance;
pub(crate) mod text_input_listener_group;
mod transition_bool_condition;
mod transition_comparator;
mod transition_condition;
mod transition_condition_op;
mod transition_duration_binding;
mod transition_focus_condition;
mod transition_input_condition;
mod transition_number_condition;
mod transition_property_comparator;
mod transition_property_viewmodel_comparator;
mod transition_trigger_condition;
mod transition_viewmodel_condition;
use animation_reset_factory::{AnimationReset, AnimationResetFactory};
#[cfg(test)]
use animation_reset_factory::{AnimationResetColorValue, AnimationResetEntry};
pub(crate) use bindables::{
    RuntimeBindableArtboard, RuntimeBindableAsset, RuntimeBindableAssetDefaultViewModelSource,
    RuntimeBindableAssetValue, RuntimeBindableBoolean, RuntimeBindableColor, RuntimeBindableEnum,
    RuntimeBindableInteger, RuntimeBindableList, RuntimeBindableNumber,
    RuntimeBindableNumberDefaultViewModelSource, RuntimeBindableString, RuntimeBindableTrigger,
    RuntimeBindableViewModel, RuntimeViewModelTrigger, StateMachineBindableArtboardInstance,
    StateMachineBindableAssetInstance, StateMachineBindableBooleanInstance,
    StateMachineBindableColorInstance, StateMachineBindableEnumInstance,
    StateMachineBindableIntegerInstance, StateMachineBindableListInstance,
    StateMachineBindableNumberInstance, StateMachineBindableStringInstance,
    StateMachineBindableTriggerInstance, StateMachineBindableViewModelInstance,
    bindable_artboard_value, bindable_asset_value, bindable_boolean_value, bindable_color_value,
    bindable_enum_value, bindable_integer_value, bindable_number_value, bindable_string_value,
    bindable_trigger_value, bindable_view_model_value, runtime_bindable_artboards,
    runtime_bindable_assets, runtime_bindable_booleans, runtime_bindable_colors,
    runtime_bindable_enums, runtime_bindable_integers, runtime_bindable_lists,
    runtime_bindable_numbers, runtime_bindable_strings, runtime_bindable_triggers,
    runtime_bindable_view_models, runtime_default_view_model_triggers,
    runtime_number_default_view_model_source_for_instance,
};
pub(crate) use blend_state_direct_instance::BlendStateDirectInstance;
use data_bind_template::{
    RuntimeStateMachineDataBindTemplate, runtime_state_machine_data_bind_templates,
};
pub use data_converter_binding::RuntimeStateMachineDataConverterBindStep;
use data_converter_binding::runtime_state_machine_data_converter_bind_steps;
#[cfg(test)]
use event_report::open_url_target;
pub use event_report::{
    StateMachineEventContext, StateMachineEventStringProperty, StateMachineReportedEvent,
};
pub use gamepad_batch::{
    GAMEPAD_BATCH_MAX_AXES, GAMEPAD_BATCH_MAX_BUTTONS, GAMEPAD_BATCH_WIRE_VERSION,
};
pub use instance::{FocusState, StateMachineInstance};
pub use layer_state::RuntimeLayerState;
pub(crate) use listener_action::{
    RuntimeScheduledListenerAction, RuntimeScheduledListenerActionExecutor,
    RuntimeScheduledListenerActionTargetsMut, perform_scheduled_listener_actions,
};
pub use listener_action_owner::RuntimeFileStateMachineActionCatalog;
pub(crate) use listener_action_owner::{RuntimeActionCoreArena, RuntimeActionCoreHandle};
#[cfg(test)]
pub(crate) use listener_bool_change::RuntimeListenerBoolChange;
#[cfg(test)]
pub(crate) use listener_input_change::RuntimeListenerInputTarget;
pub use listener_invocation::{
    ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
    ScriptListenerInvocation, ScriptPointerEventKind,
};
#[cfg(test)]
pub(crate) use listener_number_change::RuntimeListenerNumberChange;
#[cfg(test)]
pub(crate) use listener_trigger_change::RuntimeListenerTriggerChange;
pub(crate) use listener_types::RuntimeListenerType;
pub(crate) use listener_viewmodel_change::RuntimeListenerViewModelChangeValue;
pub(crate) use nested_state_machine::RuntimeNestedStateMachineInstance;
pub use nested_state_machine::RuntimeNestedStateMachineReport;
pub(crate) use scripted_listener_action::RuntimeScriptedListenerBoundValue;
pub use scripted_listener_action::RuntimeScriptedListenerDataConverterBindStep;
use scripted_listener_action::{
    RuntimeScriptedListenerActionBindingDefinition, runtime_scripted_object_binding_definition,
    runtime_scripted_object_definition,
};
use scripted_transition_condition::RuntimeScriptedTransitionCondition;
use state_instance::RuntimeStateInstance;
use state_machine::RuntimeBlendAnimationHandle;
pub use state_machine::RuntimeStateMachine;
use state_machine::next_view_model_trigger_layer_id;
pub(crate) use state_machine::{
    BlendState1DInstance, RuntimeBlendAnimation1D, RuntimeBlendAnimationDirect,
    RuntimeBlendState1D, RuntimeBlendState1DSource, RuntimeBlendStateDirect,
    RuntimeDirectBlendSource, RuntimeTransitionInterpolator, TransitionEvaluationContext,
    build_state_machines, build_state_machines_with_action_catalog,
};
pub(crate) use state_machine_fire_action::{
    RuntimeStateMachineFireAction, StateMachineFireOccurrence, perform_state_machine_fire_actions,
};
pub(crate) use state_machine_fire_trigger::RuntimeStateMachineFireTriggerPath;
use state_machine_input::runtime_state_machine_input;
pub use state_machine_input::{RuntimeStateMachineInput, StateMachineInputKind};
pub use state_machine_input_instance::StateMachineInputInstance;
#[cfg(feature = "tools")]
pub use state_machine_instance::RuntimeNestedEventChainStep;
#[cfg(any(test, feature = "tools"))]
pub use state_machine_instance::{
    RuntimeNestedEventChainPhase, RuntimeNestedEventChainTrace, RuntimeNestedNotifyBatchEntry,
    RuntimeNestedNotifyBatchTrace,
};
pub use state_machine_layer::RuntimeStateMachineLayer;
pub(crate) use state_machine_layer_instance::StateMachineLayerInstance;
pub(crate) use state_machine_listener::RuntimeStateMachineListener;
use state_machine_listener::runtime_state_machine_listener;
use state_transition::{
    RuntimeStateTransition, RuntimeStateTransitionHandle, RuntimeTransitionAnimationRef,
    TransitionAllowance, transition_duration_value,
};
use transition_bool_condition::RuntimeTransitionBoolCondition;
use transition_comparator::runtime_transition_comparators;
use transition_condition::RuntimeTransitionCondition;
use transition_condition_op::TransitionConditionOp;
use transition_duration_binding::runtime_transition_duration_bindings;
pub(crate) use transition_duration_binding::{
    RuntimeTransitionDurationBinding, StateMachineTransitionDurationInstance,
};
use transition_focus_condition::RuntimeTransitionFocusCondition;
use transition_input_condition::RuntimeTransitionInputCondition;
use transition_number_condition::RuntimeTransitionNumberCondition;
use transition_property_comparator::{
    RuntimeTransitionPropertyArtboardComparator, RuntimeTransitionPropertyComponentComparator,
};
use transition_property_viewmodel_comparator::{
    RuntimeTransitionPropertyViewModelComparator, compare_view_model_integer_pair,
};
use transition_trigger_condition::RuntimeTransitionTriggerCondition;
