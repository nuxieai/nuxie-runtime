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

mod bindables;
mod data_bind_template;
mod data_converter_binding;
mod event_report;
mod focus_action_clear;
mod focus_action_target;
mod focus_action_traversal;
mod focus_listener_group;
mod focused_input_dispatch;
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
mod state_machine_fire_action;
mod state_machine_fire_event;
mod state_machine_fire_trigger;
mod state_machine_input;
mod state_machine_input_instance;
mod state_machine_layer;
mod state_machine_layer_instance;
mod state_machine_listener;
mod state_machine_listener_single;
mod state_transition;
mod system_state_instance;
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
pub use instance::StateMachineInstance;
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
pub(crate) use state_machine_fire_action::{
    RuntimeStateMachineFireAction, StateMachineFireOccurrence, perform_state_machine_fire_actions,
};
pub(crate) use state_machine_fire_trigger::RuntimeStateMachineFireTriggerPath;
use state_machine_input::runtime_state_machine_input;
pub use state_machine_input::{RuntimeStateMachineInput, StateMachineInputKind};
pub use state_machine_input_instance::StateMachineInputInstance;
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

fn next_view_model_trigger_layer_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine {
    pub global_id: u32,
    pub name: Option<Arc<str>>,
    pub(crate) default_view_model_index: Option<usize>,
    pub inputs: Arc<Vec<Option<RuntimeStateMachineInput>>>,
    pub(crate) listeners: Arc<Vec<RuntimeStateMachineListener>>,
    pub layers: Arc<Vec<RuntimeStateMachineLayer>>,
    pub(crate) bindable_numbers: Arc<Vec<RuntimeBindableNumber>>,
    pub(crate) bindable_integers: Arc<Vec<RuntimeBindableInteger>>,
    pub(crate) bindable_colors: Arc<Vec<RuntimeBindableColor>>,
    pub(crate) bindable_strings: Arc<Vec<RuntimeBindableString>>,
    pub(crate) bindable_enums: Arc<Vec<RuntimeBindableEnum>>,
    pub(crate) bindable_assets: Arc<Vec<RuntimeBindableAsset>>,
    pub(crate) bindable_artboards: Arc<Vec<RuntimeBindableArtboard>>,
    pub(crate) bindable_lists: Arc<Vec<RuntimeBindableList>>,
    pub(crate) bindable_triggers: Arc<Vec<RuntimeBindableTrigger>>,
    pub(crate) bindable_view_models: Arc<Vec<RuntimeBindableViewModel>>,
    pub(crate) bindable_booleans: Arc<Vec<RuntimeBindableBoolean>>,
    pub(crate) view_model_triggers: Arc<Vec<RuntimeViewModelTrigger>>,
    pub(crate) transition_duration_bindings: Arc<Vec<RuntimeTransitionDurationBinding>>,
    pub(crate) data_bind_templates: Arc<Vec<RuntimeStateMachineDataBindTemplate>>,
    /// Every source `StateMachine::scriptedObjects()` occurrence in import
    /// order, including listener actions and scripted transition conditions.
    pub(crate) scripted_objects: Vec<ScriptListenerActionDefinition>,
    pub(crate) scripted_object_bindings: Vec<RuntimeScriptedListenerActionBindingDefinition>,
    pub(crate) scripted_listener_actions: Vec<ScriptListenerActionDefinition>,
    /// Pinned C++ source-StateMachine-owned generated fields for every
    /// ListenerAction and StateMachineFireAction. All concrete SMIs retain
    /// handles into this one definition arena.
    pub(crate) action_owners: RuntimeActionCoreArena,
}

impl RuntimeStateMachine {
    /// Complete state-machine `ScriptedObject` collection in imported order.
    #[doc(hidden)]
    pub fn scripted_objects(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_objects
    }

    /// Scripted listener tables that must be instantiated for each concrete
    /// [`StateMachineInstance`] occurrence.
    pub fn scripted_listener_actions(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_listener_actions
    }
}

impl RuntimeStateMachine {
    pub(crate) fn requires_post_update_state_probe(&self) -> bool {
        self.layers
            .iter()
            .flat_map(|layer| &layer.states)
            .flat_map(|state| &state.transitions)
            .flat_map(|transition| &transition.conditions)
            .any(RuntimeTransitionCondition::can_change_during_artboard_update)
    }
}

pub(crate) fn build_state_machines<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Vec<RuntimeStateMachine> {
    let action_catalog = RuntimeFileStateMachineActionCatalog::new(file);
    build_state_machines_with_action_catalog(
        file,
        graph,
        linear_animations,
        converter_cache,
        &action_catalog,
    )
}

pub(crate) fn build_state_machines_with_action_catalog<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
    action_catalog: &RuntimeFileStateMachineActionCatalog,
) -> Vec<RuntimeStateMachine> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let animation_index_by_global = linear_animations
        .iter()
        .enumerate()
        .map(|(index, animation)| (animation.global_id, index))
        .collect::<BTreeMap<_, _>>();
    let default_view_model_index = state_machine_default_view_model_index(file, artboard_index);
    let default_instance = default_view_model_index
        .and_then(|view_model_index| file.view_model_default_instance(view_model_index))
        .map(|instance| instance.object);

    file.artboard_state_machine_graphs(artboard_index)
        .into_iter()
        .map(|state_machine| {
            let action_owners = action_catalog
                .arena(state_machine.object.id)
                .expect("file action catalog must contain every accepted state machine");
            let state_machine_data_binds = state_machine.data_binds.clone();
            let bindable_numbers = runtime_bindable_numbers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_integers = runtime_bindable_integers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_colors = runtime_bindable_colors(file, &state_machine, default_instance);
            let bindable_strings = runtime_bindable_strings(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_enums = runtime_bindable_enums(file, &state_machine, default_instance);
            let bindable_assets = runtime_bindable_assets(file, &state_machine, default_instance);
            let bindable_artboards =
                runtime_bindable_artboards(file, &state_machine, default_instance);
            let bindable_lists = runtime_bindable_lists(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_triggers = runtime_bindable_triggers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_view_models = runtime_bindable_view_models(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_booleans = runtime_bindable_booleans(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let view_model_triggers =
                runtime_default_view_model_triggers(file, default_view_model_index);
            let transition_duration_bindings =
                runtime_transition_duration_bindings(file, &state_machine, default_instance);
            let data_bind_templates = runtime_state_machine_data_bind_templates(
                file,
                &state_machine,
                default_instance,
                &transition_duration_bindings,
                converter_cache,
            );
            let scripted_listener_actions = state_machine
                .scripted_objects
                .iter()
                .filter_map(|scripted| {
                    Some((
                        runtime_scripted_object_definition(
                            file,
                            scripted.object,
                            &scripted.inputs,
                        )?,
                        runtime_scripted_object_binding_definition(
                            file,
                            scripted.object,
                            &scripted.inputs,
                        )?,
                    ))
                })
                .collect::<Vec<_>>();
            let scripted_object_bindings = scripted_listener_actions
                .iter()
                .map(|(_, binding)| binding.clone())
                .collect();
            let scripted_objects = scripted_listener_actions
                .into_iter()
                .map(|(definition, _)| definition)
                .collect::<Vec<_>>();
            let scripted_listener_actions = scripted_objects
                .iter()
                .filter(|definition| {
                    definition.scripted_object_kind()
                        == crate::ScriptedStateMachineObjectKind::ListenerAction
                })
                .cloned()
                .collect();
            RuntimeStateMachine {
                global_id: state_machine.object.id,
                name: state_machine
                    .object
                    .string_property("name")
                    .map(Arc::<str>::from),
                default_view_model_index,
                inputs: Arc::new(
                    state_machine
                        .inputs
                        .iter()
                        .map(|input| input.and_then(runtime_state_machine_input))
                        .collect(),
                ),
                listeners: Arc::new(
                    state_machine
                        .listeners
                        .iter()
                        .filter_map(|listener| {
                            runtime_state_machine_listener(
                                file,
                                graph,
                                &state_machine.inputs,
                                &state_machine_data_binds,
                                listener,
                                &action_owners,
                            )
                        })
                        .collect(),
                ),
                bindable_numbers: Arc::new(bindable_numbers),
                bindable_integers: Arc::new(bindable_integers),
                bindable_colors: Arc::new(bindable_colors),
                bindable_strings: Arc::new(bindable_strings),
                bindable_enums: Arc::new(bindable_enums),
                bindable_assets: Arc::new(bindable_assets),
                bindable_artboards: Arc::new(bindable_artboards),
                bindable_lists: Arc::new(bindable_lists),
                bindable_triggers: Arc::new(bindable_triggers),
                bindable_view_models: Arc::new(bindable_view_models),
                bindable_booleans: Arc::new(bindable_booleans),
                view_model_triggers: Arc::new(view_model_triggers),
                transition_duration_bindings: Arc::new(transition_duration_bindings),
                data_bind_templates: Arc::new(data_bind_templates),
                scripted_objects,
                scripted_object_bindings,
                scripted_listener_actions,
                action_owners: action_owners.clone(),
                layers: Arc::new(
                    state_machine
                    .layers
                    .into_iter()
                    .map(|layer| {
                        let states = layer
                            .states
                            .into_iter()
                            .map(|state| {
                                let animation = state
                                    .object
                                    .filter(|object| object.type_name == "AnimationState")
                                    .map(|_| {
                                        state
                                            .animation
                                            .and_then(|animation| {
                                                animation_index_by_global
                                                    .get(&animation.id)
                                                    .copied()
                                            })
                                            .map(RuntimeLinearAnimationHandle::new)
                                            .unwrap_or_else(RuntimeLinearAnimationHandle::empty)
                                    });
                                let blend_state_1d = RuntimeBlendState1D::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                let blend_state_direct = RuntimeBlendStateDirect::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                RuntimeLayerState {
                                    global_id: state.object.map(|object| object.id),
                                    type_name: state.object.map(|object| object.type_name),
                                    animation,
                                    blend_state_1d,
                                    blend_state_direct,
                                    speed: state
                                        .object
                                        .and_then(|object| object.double_property("speed"))
                                        .unwrap_or(1.0),
                                    flags: state
                                        .object
                                        .and_then(|object| object.uint_property("flags"))
                                        .unwrap_or(0),
                                    fire_actions: state
                                        .fire_actions
                                        .iter()
                                        .map(|action| {
                                            RuntimeStateMachineFireAction::from_imported(
                                                file,
                                                action,
                                                action_owners
                                                    .handle(action.object.id)
                                                    .expect("accepted fire action has an owner"),
                                            )
                                        })
                                        .collect(),
                                    listener_actions: state
                                        .listener_actions
                                        .iter()
                                        .map(|action| {
                                            RuntimeScheduledListenerAction::from_imported(
                                                file,
                                                graph,
                                                &state_machine.inputs,
                                                &state_machine_data_binds,
                                                action,
                                                action_owners
                                                    .handle(action.object.id)
                                                    .expect(
                                                        "accepted listener action has an owner",
                                                    ),
                                            )
                                        })
                                        .collect(),
                                    transitions: state
                                        .transitions
                                        .into_iter()
                                        .map(|transition| {
                                            let interpolator = transition.interpolator.and_then(
                                                RuntimeTransitionInterpolator::from_object,
                                            );
                                            let conditions = transition
                                                .conditions
                                                .iter()
                                                .filter_map(|condition| {
                                                    RuntimeTransitionCondition::from_object(
                                                        file,
                                                        graph,
                                                        &state_machine.inputs,
                                                        condition,
                                                    )
                                                })
                                                .collect::<Vec<_>>();
                                            let direct_input_conditions_only = conditions
                                                .iter()
                                                .all(RuntimeTransitionCondition::is_direct_input);
                                            RuntimeStateTransition {
                                                global_id: transition.object.id,
                                                state_to_index: transition.state_to_index,
                                                exit_blend_animation_index: transition
                                                    .exit_blend_animation_index,
                                                duration: transition
                                                    .object
                                                    .uint_property("duration")
                                                    .unwrap_or(0),
                                                exit_time: transition
                                                    .object
                                                    .uint_property("exitTime")
                                                    .unwrap_or(0),
                                                flags: transition
                                                    .object
                                                    .uint_property("flags")
                                                    .unwrap_or(0),
                                                random_weight: transition
                                                    .object
                                                    .uint_property("randomWeight")
                                                    .unwrap_or(1)
                                                    as u32,
                                                conditions,
                                                direct_input_conditions_only,
                                                fire_actions: transition
                                                    .fire_actions
                                                    .iter()
                                                    .map(|action| {
                                                        RuntimeStateMachineFireAction::from_imported(
                                                            file,
                                                            action,
                                                            action_owners
                                                                .handle(action.object.id)
                                                                .expect(
                                                                    "accepted fire action has an owner",
                                                                ),
                                                        )
                                                    })
                                                    .collect(),
                                                listener_actions: transition
                                                    .listener_actions
                                                    .iter()
                                                    .map(|action| {
                                                        RuntimeScheduledListenerAction::from_imported(
                                                            file,
                                                            graph,
                                                            &state_machine.inputs,
                                                            &state_machine_data_binds,
                                                            action,
                                                            action_owners
                                                                .handle(action.object.id)
                                                                .expect(
                                                                    "accepted listener action has an owner",
                                                                ),
                                                        )
                                                    })
                                                    .collect(),
                                                interpolator,
                                                has_unsupported_interpolator: transition
                                                    .interpolator
                                                    .is_some()
                                                    && interpolator.is_none(),
                                            }
                                        })
                                        .collect(),
                                }
                            })
                            .collect::<Vec<_>>();
                        let (entry_state_index, any_state_index, exit_state_index) =
                            RuntimeStateMachineLayer::resolve_system_state_indices(&states);
                        RuntimeStateMachineLayer {
                            global_id: layer.object.id,
                            name: layer.object.string_property("name").map(ToOwned::to_owned),
                            states,
                            entry_state_index,
                            any_state_index,
                            exit_state_index,
                        }
                    })
                    .collect(),
                ),
            }
        })
        .collect()
}

fn state_machine_default_view_model_index(
    file: &RuntimeFile,
    artboard_index: usize,
) -> Option<usize> {
    file.resolved_view_model_for_artboard(artboard_index)
        .map(|view_model| view_model.view_model_index)
        .or_else(|| file.view_model(0).map(|_| 0))
}

pub(crate) struct TransitionEvaluationContext<'a> {
    bindable_numbers: &'a [StateMachineBindableNumberInstance],
    bindable_integers: &'a [StateMachineBindableIntegerInstance],
    bindable_colors: &'a [StateMachineBindableColorInstance],
    bindable_strings: &'a [StateMachineBindableStringInstance],
    bindable_enums: &'a [StateMachineBindableEnumInstance],
    bindable_assets: &'a [StateMachineBindableAssetInstance],
    bindable_artboards: &'a [StateMachineBindableArtboardInstance],
    bindable_triggers: &'a [StateMachineBindableTriggerInstance],
    bindable_view_models: &'a [StateMachineBindableViewModelInstance],
    bindable_booleans: &'a [StateMachineBindableBooleanInstance],
    data_context_present: bool,
    layer_index: usize,
    view_model_trigger_layer_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeTransitionInterpolator {
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

impl RuntimeTransitionInterpolator {
    pub(crate) fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    pub(crate) fn transform(self, factor: f32) -> f32 {
        match self {
            Self::CubicEase { x1, y1, x2, y2 } => {
                RuntimeInterpolator::CubicEase { x1, y1, x2, y2 }.transform(factor)
            }
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => RuntimeInterpolator::Elastic {
                amplitude,
                period,
                easing_value,
            }
            .transform(factor),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendState1D {
    pub(crate) source: RuntimeBlendState1DSource,
    pub(crate) animations: Vec<RuntimeBlendAnimation1D>,
}

impl RuntimeBlendState1D {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state: &nuxie_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        let source = match object.type_name {
            "BlendState1DInput" => RuntimeBlendState1DSource::Input {
                input_index: object
                    .uint_property("inputId")
                    .filter(|input_id| *input_id != u64::from(u32::MAX))
                    .and_then(|input_id| usize::try_from(input_id).ok()),
            },
            "BlendState1DViewModel" => RuntimeBlendState1DSource::BindableProperty {
                global_id: file
                    .latest_bindable_property_for_object(object)
                    .map(|property| property.id as u32),
            },
            _ => return None,
        };
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                if animation.object.type_name != "BlendAnimation1D" {
                    return None;
                }
                let definition = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())
                    .map(RuntimeLinearAnimationHandle::new)
                    .unwrap_or_else(RuntimeLinearAnimationHandle::empty);
                Some(RuntimeBlendAnimation1D {
                    animation: definition,
                    value: animation.object.double_property("value").unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();
        Some(Self { source, animations })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeBlendState1DSource {
    Input { input_index: Option<usize> },
    BindableProperty { global_id: Option<u32> },
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimation1D {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendStateDirect {
    pub(crate) animations: Vec<RuntimeBlendAnimationDirect>,
}

impl RuntimeBlendStateDirect {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state: &nuxie_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        if object.type_name != "BlendStateDirect" {
            return None;
        }
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                if animation.object.type_name != "BlendAnimationDirect" {
                    return None;
                }
                let definition = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())
                    .map(RuntimeLinearAnimationHandle::new)
                    .unwrap_or_else(RuntimeLinearAnimationHandle::empty);
                Some(RuntimeBlendAnimationDirect {
                    animation: definition,
                    source: RuntimeDirectBlendSource::from_object(file, animation.object),
                })
            })
            .collect::<Vec<_>>();
        Some(Self { animations })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimationDirect {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) source: RuntimeDirectBlendSource,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeDirectBlendSource {
    Input { input_index: usize },
    MixValue { value: f32 },
    BindableProperty { global_id: Option<u32> },
}

impl RuntimeDirectBlendSource {
    fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Self {
        match object.uint_property("blendSource").unwrap_or(0) {
            1 => Self::MixValue {
                value: object.double_property("mixValue").unwrap_or(100.0),
            },
            2 => Self::BindableProperty {
                global_id: file
                    .latest_bindable_property_for_object(object)
                    .map(|property| property.id as u32),
            },
            _ => Self::Input {
                input_index: object
                    .uint_property("inputId")
                    .and_then(|input_id| usize::try_from(input_id).ok())
                    .unwrap_or(usize::MAX),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeBlendAnimationHandle(usize);

impl RuntimeBlendAnimationHandle {
    fn new(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BlendState1DInstance {
    animations: Vec<BlendAnimation1DInstance>,
    from: Option<RuntimeBlendAnimationHandle>,
    to: Option<RuntimeBlendAnimationHandle>,
    animation_reset: Option<AnimationReset>,
}

impl BlendState1DInstance {
    pub(crate) fn new(
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        reset_blend_values: bool,
    ) -> Self {
        let animations = blend_state
            .animations
            .iter()
            .enumerate()
            .filter_map(|(definition_index, animation)| {
                let linear_animation = artboard.linear_animation_definition(animation.animation)?;
                Some(BlendAnimation1DInstance {
                    definition: RuntimeBlendAnimationHandle::new(definition_index),
                    animation: LinearAnimationInstance::new(
                        animation.animation,
                        linear_animation,
                        1.0,
                    ),
                    mix: 0.0,
                })
            })
            .collect();
        let animation_reset = if reset_blend_values {
            let animation_indices = blend_state
                .animations
                .iter()
                .map(|animation| animation.animation.index())
                .collect::<Vec<_>>();
            Some(AnimationResetFactory::from_animation_indices(
                artboard,
                &animation_indices,
                true,
            ))
        } else {
            None
        };

        Self {
            animations,
            from: None,
            to: None,
            animation_reset,
        }
    }

    pub(crate) fn advance(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            blend_state,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                if let Some(events) = reported_events.as_mut() {
                    artboard.advance_linear_animation_instance_with_events(
                        &mut animation.animation,
                        elapsed_seconds,
                        *events,
                    );
                } else {
                    artboard.advance_linear_animation_instance(
                        &mut animation.animation,
                        elapsed_seconds,
                    );
                }
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        if self.animations.is_empty() {
            return;
        }

        let value = match blend_state.source {
            RuntimeBlendState1DSource::Input { input_index } => input_index
                .and_then(|input_index| inputs.get(input_index))
                .and_then(StateMachineInputInstance::number_value)
                .unwrap_or(0.0),
            RuntimeBlendState1DSource::BindableProperty { global_id } => global_id
                .and_then(|global_id| bindable_number_value(bindable_numbers, global_id))
                .unwrap_or(0.0),
        };

        let to_index = self.animation_index(blend_state, value);
        self.to = (to_index < self.animations.len()).then(|| self.animations[to_index].definition);
        self.from = to_index
            .checked_sub(1)
            .and_then(|index| self.animations.get(index))
            .map(|animation| animation.definition);
        let to_value = self
            .to
            .and_then(|handle| blend_state.animations.get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let from_value = self
            .from
            .and_then(|handle| blend_state.animations.get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let (mix, mix_from) = if self.to.is_none() || self.from.is_none() || to_value == from_value
        {
            (1.0, 1.0)
        } else {
            let mix = (value - from_value) / (to_value - from_value);
            (mix, 1.0 - mix)
        };

        for animation in &mut self.animations {
            let animation_value = blend_state
                .animations
                .get(animation.definition.index())
                .map(|definition| definition.value)
                .unwrap_or(0.0);
            if self.to.is_some() && animation_value == to_value {
                animation.mix = mix;
            } else if self.from.is_some() && animation_value == from_value {
                animation.mix = mix_from;
            } else {
                animation.mix = 0.0;
            }
        }
    }

    fn animation_index(&self, blend_state: &RuntimeBlendState1D, value: f32) -> usize {
        let mut index = 0_usize;
        let mut start = 0_isize;
        let mut end = self.animations.len() as isize - 1;

        while start <= end {
            let mid = (start + end) >> 1;
            let closest_value = self
                .animations
                .get(mid as usize)
                .and_then(|animation| blend_state.animations.get(animation.definition.index()))
                .map(|animation| animation.value)
                .unwrap_or(0.0);
            if closest_value < value {
                start = mid + 1;
            } else if closest_value > value {
                end = mid - 1;
            } else {
                index = mid as usize;
                break;
            }

            index = start as usize;
        }

        index
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .iter()
            .find(|animation| animation.definition.index() == index)
            .map(|animation| &animation.animation)
    }

    fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animations {
            callback(&mut animation.animation);
        }
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        if let Some(reset) = self.animation_reset.as_ref() {
            changed |= reset.apply(artboard);
        }
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |= animation.animation.apply(artboard, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimation1DInstance {
    definition: RuntimeBlendAnimationHandle,
    animation: LinearAnimationInstance,
    mix: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct BlendStateDirectInstance {
    animations: Vec<BlendAnimationDirectInstance>,
}

impl BlendStateDirectInstance {
    pub(crate) fn new(blend_state: &RuntimeBlendStateDirect, artboard: &ArtboardInstance) -> Self {
        let animations = blend_state
            .animations
            .iter()
            .enumerate()
            .filter_map(|(definition_index, animation)| {
                let linear_animation = artboard.linear_animation_definition(animation.animation)?;
                Some(BlendAnimationDirectInstance {
                    definition: RuntimeBlendAnimationHandle::new(definition_index),
                    animation: LinearAnimationInstance::new(
                        animation.animation,
                        linear_animation,
                        1.0,
                    ),
                    mix: 0.0,
                })
            })
            .collect();

        Self { animations }
    }

    pub(crate) fn advance(
        &mut self,
        blend_state: &RuntimeBlendStateDirect,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        blend_state: &RuntimeBlendStateDirect,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            blend_state,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
        blend_state: &RuntimeBlendStateDirect,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                if let Some(events) = reported_events.as_mut() {
                    artboard.advance_linear_animation_instance_with_events(
                        &mut animation.animation,
                        elapsed_seconds,
                        *events,
                    );
                } else {
                    artboard.advance_linear_animation_instance(
                        &mut animation.animation,
                        elapsed_seconds,
                    );
                }
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        blend_state: &RuntimeBlendStateDirect,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        for animation in &mut self.animations {
            let Some(definition) = blend_state.animations.get(animation.definition.index()) else {
                continue;
            };
            let value = match definition.source {
                RuntimeDirectBlendSource::Input { input_index } => inputs
                    .get(input_index)
                    .and_then(StateMachineInputInstance::number_value)
                    .unwrap_or(0.0),
                RuntimeDirectBlendSource::MixValue { value } => value,
                RuntimeDirectBlendSource::BindableProperty { global_id } => {
                    let Some(value) = global_id
                        .and_then(|global_id| bindable_number_value(bindable_numbers, global_id))
                    else {
                        // C++ leaves the current mix untouched when the authored
                        // bindable property cannot produce a number instance.
                        continue;
                    };
                    value
                }
            };
            animation.mix = (value / 100.0).clamp(0.0, 1.0);
        }
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .iter()
            .find(|animation| animation.definition.index() == index)
            .map(|animation| &animation.animation)
    }

    fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animations {
            callback(&mut animation.animation);
        }
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |= animation.animation.apply(artboard, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimationDirectInstance {
    definition: RuntimeBlendAnimationHandle,
    animation: LinearAnimationInstance,
    mix: f32,
}

#[derive(Debug, Clone)]
struct AnimationReset {
    // `StateMachineInstance::clone` is a Rust snapshot API with no C++
    // occurrence-copy counterpart. Share this immutable reset lease so a
    // snapshot never clones factory state; the final Arc owner returns the
    // cleared storage to the C++-shaped global pool.
    storage: Arc<AnimationResetStorage>,
}

#[derive(Debug)]
struct AnimationResetStorage {
    entries: Vec<AnimationResetEntry>,
}

impl Drop for AnimationResetStorage {
    fn drop(&mut self) {
        let mut entries = std::mem::take(&mut self.entries);
        entries.clear();
        animation_reset_pool()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(entries);
    }
}

#[derive(Debug)]
enum AnimationResetEntry {
    Double {
        local_id: usize,
        property_key: u16,
        transform_property: Option<TransformProperty>,
        value: f32,
    },
    Color {
        local_id: usize,
        property_key: u16,
        solid_color_property: bool,
        data_bind_observed: bool,
        value: AnimationResetColorValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationResetColorValue {
    /// Pinned C++ serializes the signed color through `float` and converts the
    /// decoded float back to `int` in `CoreRegistry::setColor`.
    DefinedFloat(f32),
    /// A positive `int` close enough to `INT_MAX` rounds to 2^31 as `float`;
    /// converting that value back to `int` is undefined in C++. Preserve the
    /// serialized float and apply the project's explicit saturating conversion
    /// decision instead of attempting to emulate undefined behavior.
    SaturatingFloatToInt(f32),
}

impl AnimationResetColorValue {
    fn from_color(value: u32) -> Self {
        let encoded = (value as i32) as f32;
        if encoded < 2_147_483_648.0 {
            Self::DefinedFloat(encoded)
        } else {
            Self::SaturatingFloatToInt(encoded)
        }
    }

    fn replay(self) -> u32 {
        match self {
            Self::DefinedFloat(value) => (value as i32) as u32,
            // Project divergence D2 binds Rust's saturating conversion where
            // the corresponding C++ float-to-int conversion is undefined.
            Self::SaturatingFloatToInt(value) => (value as i32) as u32,
        }
    }
}

#[derive(Debug)]
struct AnimationResetObjectData {
    local_id: usize,
    property_keys: BTreeSet<u16>,
    entries: Vec<AnimationResetEntry>,
}

impl AnimationResetObjectData {
    fn new(local_id: usize) -> Self {
        Self {
            local_id,
            property_keys: BTreeSet::new(),
            entries: Vec::new(),
        }
    }
}

struct AnimationResetFactory;

fn animation_reset_pool() -> &'static Mutex<Vec<Vec<AnimationResetEntry>>> {
    static POOL: OnceLock<Mutex<Vec<Vec<AnimationResetEntry>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(Vec::new()))
}

impl AnimationResetFactory {
    fn from_animation_indices(
        artboard: &ArtboardInstance,
        animation_indices: &[usize],
        use_first_as_baseline: bool,
    ) -> AnimationReset {
        let mut entries = animation_reset_pool()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop()
            .unwrap_or_default();
        debug_assert!(entries.is_empty());
        let mut objects = Vec::<AnimationResetObjectData>::new();

        for (animation_order, animation_index) in animation_indices.iter().enumerate() {
            let Some(animation) = artboard.linear_animation(*animation_index) else {
                continue;
            };
            let use_baseline = use_first_as_baseline && animation_order == 0;
            for keyed_object in animation.keyed_objects.iter() {
                let object_index = objects
                    .iter()
                    .position(|object| object.local_id == keyed_object.target_local_id)
                    .unwrap_or_else(|| {
                        objects.push(AnimationResetObjectData::new(keyed_object.target_local_id));
                        objects.len() - 1
                    });
                let object = &mut objects[object_index];
                for keyed_property in &keyed_object.keyed_properties {
                    match &keyed_property.target {
                        RuntimeKeyedPropertyTarget::Double { transform_property } => {
                            if !object.property_keys.insert(keyed_property.property_key) {
                                continue;
                            }
                            let value = if use_baseline {
                                keyed_property.first_double_value()
                            } else {
                                current_animation_reset_double_value(
                                    artboard,
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                    *transform_property,
                                )
                            };
                            if let Some(value) = value {
                                object.entries.push(AnimationResetEntry::Double {
                                    local_id: keyed_object.target_local_id,
                                    property_key: keyed_property.property_key,
                                    transform_property: *transform_property,
                                    value,
                                });
                            }
                        }
                        RuntimeKeyedPropertyTarget::Color {
                            solid_color_property,
                            data_bind_observed,
                        } => {
                            if !object.property_keys.insert(keyed_property.property_key) {
                                continue;
                            }
                            let value = if use_baseline {
                                keyed_property.first_color_value()
                            } else if *solid_color_property {
                                artboard.solid_color_value(keyed_object.target_local_id)
                            } else {
                                artboard.color_property(
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                )
                            };
                            if let Some(value) = value {
                                object.entries.push(AnimationResetEntry::Color {
                                    local_id: keyed_object.target_local_id,
                                    property_key: keyed_property.property_key,
                                    solid_color_property: *solid_color_property,
                                    data_bind_observed: *data_bind_observed,
                                    value: AnimationResetColorValue::from_color(value),
                                });
                            }
                        }
                        RuntimeKeyedPropertyTarget::Bool
                        | RuntimeKeyedPropertyTarget::Uint
                        | RuntimeKeyedPropertyTarget::String
                        | RuntimeKeyedPropertyTarget::Callback { .. } => {}
                    }
                }
            }
        }

        for object in objects {
            entries.extend(object.entries);
        }
        AnimationReset {
            storage: Arc::new(AnimationResetStorage { entries }),
        }
    }
}

impl AnimationReset {
    fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        for entry in &self.storage.entries {
            match entry {
                AnimationResetEntry::Double {
                    local_id,
                    property_key,
                    transform_property,
                    value,
                } => {
                    changed |= match transform_property {
                        Some(transform_property) => artboard.set_transform_property_with_key(
                            *local_id,
                            *transform_property,
                            *property_key,
                            *value,
                        ),
                        None => {
                            artboard.set_keyed_double_property(*local_id, *property_key, *value)
                        }
                    };
                }
                AnimationResetEntry::Color {
                    local_id,
                    property_key,
                    solid_color_property,
                    data_bind_observed,
                    value,
                } => {
                    changed |= if *solid_color_property {
                        artboard.set_keyed_solid_color_property(
                            *local_id,
                            *property_key,
                            *data_bind_observed,
                            value.replay(),
                        )
                    } else {
                        artboard.set_keyed_color_property(*local_id, *property_key, value.replay())
                    };
                }
            }
        }
        changed
    }
}

fn current_animation_reset_double_value(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
    transform_property: Option<TransformProperty>,
) -> Option<f32> {
    if let Some(property) = transform_property {
        artboard.transform_property(local_id, property)
    } else {
        artboard.double_property(local_id, property_key)
    }
}

#[cfg(test)]
mod animation_tests {
    use super::*;
    use crate::view_model::RuntimeFontAssetValue;

    #[test]
    fn listener_asset_clone_retains_live_font_payload() {
        let live: Arc<[u8]> = vec![1, 3, 5, 7].into();
        let mut font = RuntimeFontAssetValue::default();
        assert!(font.set_live_font_bytes(Some(Arc::clone(&live))));
        let action = RuntimeScheduledListenerAction::ViewModelChange(
            listener_viewmodel_change::RuntimeListenerViewModelChange::for_test(
                0,
                Some(4),
                Some(RuntimeListenerViewModelChangeValue::Asset(
                    RuntimeBindableAssetValue::from_font_value(font),
                )),
            ),
        );

        let RuntimeScheduledListenerAction::ViewModelChange(
            listener_viewmodel_change::RuntimeListenerViewModelChange {
                value: Some(RuntimeListenerViewModelChangeValue::Asset(value)),
                ..
            },
        ) = action.clone()
        else {
            panic!("listener action lost its asset value");
        };
        assert_eq!(
            value.asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert!(
            value
                .font_value()
                .and_then(RuntimeFontAssetValue::live_font_bytes_arc)
                .is_some_and(|value| Arc::ptr_eq(value, &live)),
            "cloning a scheduled listener must retain the same live font"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{
        AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile, read_runtime_file,
    };
    use nuxie_graph::GraphFile;
    use std::path::PathBuf;

    fn rive_runtime_fixture(name: &str) -> PathBuf {
        PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(name)
    }

    #[test]
    fn blend_occurrences_retain_definition_handles_and_shared_empty_animation() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation fixture"),
        )
        .expect("import animation fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation artboard");
        assert!(!artboard.linear_animations().is_empty());

        let blend_state = RuntimeBlendState1D {
            source: RuntimeBlendState1DSource::Input {
                input_index: Some(0),
            },
            animations: vec![
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::new(0),
                    value: 0.0,
                },
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::empty(),
                    value: 100.0,
                },
            ],
        };
        let input_definitions = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("blend".to_owned()),
            25.0,
        ))]);
        let inputs = vec![StateMachineInputInstance::new(0, input_definitions)];
        let mut occurrence = BlendState1DInstance::new(&blend_state, &artboard, false);

        assert_eq!(occurrence.animations.len(), blend_state.animations.len());
        assert_eq!(
            occurrence.animations[0].definition,
            RuntimeBlendAnimationHandle::new(0)
        );
        assert_eq!(
            occurrence.animations[1].definition,
            RuntimeBlendAnimationHandle::new(1)
        );
        let empty = artboard
            .linear_animation_instance_definition(&occurrence.animations[1].animation)
            .expect("shared empty definition");
        assert!(std::ptr::eq(
            empty,
            artboard.empty_linear_animation.as_ref()
        ));

        occurrence.advance(&blend_state, &artboard, &inputs, &[], 0.0);
        assert_eq!(occurrence.from, Some(RuntimeBlendAnimationHandle::new(0)));
        assert_eq!(occurrence.to, Some(RuntimeBlendAnimationHandle::new(1)));
        assert_eq!(occurrence.animations[0].mix, 0.75);
        assert_eq!(occurrence.animations[1].mix, 0.25);

        let mut direct_state = RuntimeBlendStateDirect {
            animations: vec![RuntimeBlendAnimationDirect {
                animation: RuntimeLinearAnimationHandle::empty(),
                source: RuntimeDirectBlendSource::MixValue { value: 200.0 },
            }],
        };
        let mut direct_occurrence = BlendStateDirectInstance::new(&direct_state, &artboard);
        direct_state.animations[0].source = RuntimeDirectBlendSource::MixValue { value: 40.0 };
        direct_occurrence.advance(&direct_state, &artboard, &[], &[], 0.0);
        assert_eq!(
            direct_occurrence.animations[0].definition,
            RuntimeBlendAnimationHandle::new(0)
        );
        assert_eq!(direct_occurrence.animations[0].mix, 0.4);

        let empty_state = RuntimeLayerState {
            global_id: Some(2),
            type_name: Some("AnimationState"),
            animation: Some(RuntimeLinearAnimationHandle::empty()),
            blend_state_1d: None,
            blend_state_direct: None,
            speed: 1.0,
            flags: 0,
            fire_actions: Vec::new(),
            listener_actions: Vec::new(),
            transitions: Vec::new(),
        };
        let layer = RuntimeStateMachineLayer {
            global_id: 3,
            name: None,
            states: vec![empty_state],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        };
        let layer_occurrence = StateMachineLayerInstance::new(&layer, &artboard, &[], &[], &[]);
        let empty_state_animation = layer_occurrence
            .current_animation()
            .expect("AnimationState always creates an animation occurrence");
        assert!(std::ptr::eq(
            artboard
                .linear_animation_instance_definition(empty_state_animation)
                .expect("AnimationState empty definition"),
            artboard.empty_linear_animation.as_ref()
        ));
        assert!(!empty_state_animation.apply(&mut artboard, 1.0));
    }

    #[test]
    fn scripted_listener_action_resolves_non_module_script_asset_by_file_ordinal() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
                .expect("read scripted listener fixture"),
        )
        .expect("import scripted listener fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let artboard = graph.artboards.first().expect("fixture artboard");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let state_machines = build_state_machines(&file, artboard, &[], &mut converter_cache);
        let action = state_machines
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions()
            .first()
            .expect("scripted listener action");

        assert_eq!(action.action_global_id(), 55);
        assert_eq!(action.asset_ordinal(), 0);
        assert_eq!(action.asset_name(), "ListenerActionAppend");
    }

    #[test]
    fn scripted_listener_action_retains_module_asset_as_inert() {
        let mut file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
                .expect("read scripted listener fixture"),
        )
        .expect("import scripted listener fixture");
        file.objects
            .get_mut(1)
            .and_then(Option::as_mut)
            .expect("first ScriptAsset")
            .properties
            .push(nuxie_binary::RuntimeProperty {
                key: 914,
                name: "isModule",
                owner: "ScriptAsset",
                value: nuxie_binary::FieldValue::Bool(true),
            });
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let state_machines = build_state_machines(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &[],
            &mut converter_cache,
        );

        let actions = state_machines
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_global_id(), 55);
        assert_eq!(actions[0].asset_ordinal(), 0);
        assert_eq!(actions[0].asset_name(), "ListenerActionAppend");
        assert!(
            !actions[0].has_protocol_asset(),
            "C++ retains the action and its inputs but module ScriptAssets have no protocol generator"
        );
    }

    #[test]
    fn scripted_listener_action_retains_missing_out_of_range_and_wrong_assets() {
        fn actions(file: &RuntimeFile) -> Vec<ScriptListenerActionDefinition> {
            let graph = GraphFile::from_runtime_file(file).expect("build fixture graph");
            let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
            build_state_machines(
                file,
                graph.artboards.first().expect("fixture artboard"),
                &[],
                &mut converter_cache,
            )
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions()
            .to_vec()
        }

        fn action_mut(file: &mut RuntimeFile) -> &mut RuntimeObject {
            file.objects
                .iter_mut()
                .flatten()
                .find(|object| object.type_name == "ScriptedListenerAction")
                .expect("fixture ScriptedListenerAction")
        }

        let bytes = std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
            .expect("read scripted listener fixture");
        let baseline_file = read_runtime_file(&bytes).expect("import baseline fixture");
        let baseline = actions(&baseline_file);
        assert_eq!(baseline.len(), 1);
        let baseline_input_ids = baseline[0]
            .inputs()
            .iter()
            .map(|input| input.input_global_id())
            .collect::<Vec<_>>();

        for case in ["missing", "out-of-range", "wrong-type"] {
            let mut file = read_runtime_file(&bytes).expect("import mutated fixture");
            match case {
                "missing" => {
                    action_mut(&mut file)
                        .properties
                        .retain(|property| property.name != "scriptAssetId");
                }
                "out-of-range" => {
                    let property = action_mut(&mut file)
                        .properties
                        .iter_mut()
                        .find(|property| property.name == "scriptAssetId")
                        .expect("fixture scriptAssetId");
                    property.value = nuxie_binary::FieldValue::Uint(999);
                }
                "wrong-type" => {
                    file.objects
                        .iter_mut()
                        .flatten()
                        .find(|object| object.type_name == "ScriptAsset")
                        .expect("fixture ScriptAsset")
                        .type_name = "ImageAsset";
                }
                _ => unreachable!(),
            }

            let retained = actions(&file);
            assert_eq!(retained.len(), 1, "{case}");
            assert_eq!(retained[0].action_global_id(), 55, "{case}");
            assert!(!retained[0].has_protocol_asset(), "{case}");
            assert_eq!(
                retained[0]
                    .inputs()
                    .iter()
                    .map(|input| input.input_global_id())
                    .collect::<Vec<_>>(),
                baseline_input_ids,
                "{case}: the authored input occurrence list must survive unchanged"
            );
        }
    }

    #[test]
    fn scheduled_listener_batch_keeps_scripted_actions_in_authored_order() {
        struct RecordingExecutor {
            reported_event_counts: Vec<usize>,
            fail: bool,
        }

        impl RuntimeScheduledListenerActionExecutor for RecordingExecutor {
            fn perform_instance_action(
                &mut self,
                _artboard: &mut ArtboardInstance,
                action: &RuntimeScheduledListenerAction,
                targets: RuntimeScheduledListenerActionTargetsMut<'_>,
            ) -> Result<bool, ScriptError> {
                assert!(matches!(
                    action,
                    RuntimeScheduledListenerAction::Scripted { .. }
                ));
                self.reported_event_counts
                    .push(targets.reported_events.len());
                if self.fail {
                    return Err(ScriptError::new("scheduled listener failed"));
                }
                Ok(true)
            }
        }

        let type_key = |name: &str| {
            nuxie_schema::definition_by_name(name)
                .unwrap_or_else(|| panic!("missing schema definition {name}"))
                .type_key
                .int
        };
        let parent = |owner: &str, value: u64| AuthoringProperty {
            key: property_key_for_name(owner, "parentId").expect("parentId property"),
            value: AuthoringValue::Uint(value),
        };
        let file = RuntimeFile::from_authoring_records(vec![
            AuthoringRecord {
                type_key: type_key("Backboard"),
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: type_key("Artboard"),
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: type_key("Event"),
                properties: vec![parent("Event", 0)],
            },
            AuthoringRecord {
                type_key: type_key("Event"),
                properties: vec![parent("Event", 0)],
            },
        ])
        .expect("import two live event occurrences");
        let actions = vec![
            RuntimeScheduledListenerAction::FireEvent(
                listener_fire_event::RuntimeListenerFireEvent::for_test(
                    StateMachineFireOccurrence::AtStart.value(),
                    Some(1),
                ),
            ),
            RuntimeScheduledListenerAction::scripted_for_test(
                StateMachineFireOccurrence::AtStart.value(),
                Some(ScriptListenerActionDefinition::new(
                    44,
                    2,
                    "action".to_owned(),
                )),
            ),
            RuntimeScheduledListenerAction::FireEvent(
                listener_fire_event::RuntimeListenerFireEvent::for_test(
                    StateMachineFireOccurrence::AtStart.value(),
                    Some(2),
                ),
            ),
        ];
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate listener artboard");
        let mut reported_events = Vec::new();
        let mut executor = RecordingExecutor {
            reported_event_counts: Vec::new(),
            fail: false,
        };

        assert!(
            perform_scheduled_listener_actions(
                &actions,
                StateMachineFireOccurrence::AtStart,
                &mut artboard,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut [],
                    reported_events: &mut reported_events,
                    bindable_numbers: &mut [],
                    bindable_integers: &mut [],
                    bindable_colors: &mut [],
                    bindable_strings: &mut [],
                    bindable_enums: &mut [],
                    bindable_assets: &mut [],
                    bindable_artboards: &mut [],
                    bindable_lists: &mut [],
                    bindable_triggers: &mut [],
                    bindable_view_models: &mut [],
                    bindable_booleans: &mut [],
                    transition_durations: &mut [],
                },
                &mut executor,
            )
            .expect("execute scheduled listener actions")
        );
        assert_eq!(executor.reported_event_counts, [1]);
        assert_eq!(
            reported_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [1, 2]
        );

        reported_events.clear();
        executor.fail = true;
        assert!(
            perform_scheduled_listener_actions(
                &actions,
                StateMachineFireOccurrence::AtStart,
                &mut artboard,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut [],
                    reported_events: &mut reported_events,
                    bindable_numbers: &mut [],
                    bindable_integers: &mut [],
                    bindable_colors: &mut [],
                    bindable_strings: &mut [],
                    bindable_enums: &mut [],
                    bindable_assets: &mut [],
                    bindable_artboards: &mut [],
                    bindable_lists: &mut [],
                    bindable_triggers: &mut [],
                    bindable_view_models: &mut [],
                    bindable_booleans: &mut [],
                    transition_durations: &mut [],
                },
                &mut executor,
            )
            .expect("script failure is consumed and the authored action tail continues")
        );
        assert_eq!(
            reported_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn reported_event_metadata_preserves_open_url_values_and_ordinary_absence() {
        assert_eq!(open_url_target(0), "_blank");
        assert_eq!(open_url_target(1), "_parent");
        assert_eq!(open_url_target(2), "_self");
        assert_eq!(open_url_target(3), "_top");
        assert_eq!(open_url_target(4), "");
        assert_eq!(open_url_target(u64::MAX), "");

        let fixture = rive_runtime_fixture("event_on_listener.riv");
        let file = read_runtime_file(&std::fs::read(fixture).expect("read event fixture"))
            .expect("import event fixture");
        let open_url = file
            .objects
            .iter()
            .flatten()
            .find(|object| {
                object.type_name == "OpenUrlEvent"
                    && object.string_property("url") == Some("http://rive.app/delete-me")
            })
            .expect("authored OpenURL event");
        let open_url = StateMachineReportedEvent::from_runtime_event(7, open_url);
        assert_eq!(open_url.url(), Some("http://rive.app/delete-me"));
        assert_eq!(open_url.target(), Some("_blank"));

        let ordinary = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "Event")
            .expect("ordinary event");
        let ordinary = StateMachineReportedEvent::from_runtime_event(8, ordinary);
        assert_eq!(ordinary.url(), None);
        assert_eq!(ordinary.target(), None);
    }

    #[test]
    fn animation_reset_retains_first_seen_owner_order_and_shares_one_pool_lease() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation-reset fixture"),
        )
        .expect("import animation-reset fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation-reset graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation-reset artboard");
        let animation_indices = (0..artboard.linear_animations().len()).collect::<Vec<_>>();

        let reset =
            AnimationResetFactory::from_animation_indices(&artboard, &animation_indices, false);
        let cloned = reset.clone();
        assert!(
            Arc::ptr_eq(&reset.storage, &cloned.storage),
            "Rust snapshot clones must share one factory-owned reset lease"
        );

        let actual = reset
            .storage
            .entries
            .iter()
            .map(|entry| match entry {
                AnimationResetEntry::Double {
                    local_id,
                    property_key,
                    ..
                }
                | AnimationResetEntry::Color {
                    local_id,
                    property_key,
                    ..
                } => (*local_id, *property_key),
            })
            .collect::<Vec<_>>();
        let mut expected_objects = Vec::<(usize, BTreeSet<u16>, Vec<u16>)>::new();
        for animation in artboard.linear_animations() {
            for keyed_object in animation.keyed_objects.iter() {
                let object_index = expected_objects
                    .iter()
                    .position(|(local_id, _, _)| *local_id == keyed_object.target_local_id)
                    .unwrap_or_else(|| {
                        expected_objects.push((
                            keyed_object.target_local_id,
                            BTreeSet::new(),
                            Vec::new(),
                        ));
                        expected_objects.len() - 1
                    });
                let (_, seen, properties) = &mut expected_objects[object_index];
                for keyed_property in &keyed_object.keyed_properties {
                    if matches!(
                        &keyed_property.target,
                        RuntimeKeyedPropertyTarget::Double { .. }
                            | RuntimeKeyedPropertyTarget::Color { .. }
                    ) && seen.insert(keyed_property.property_key)
                    {
                        properties.push(keyed_property.property_key);
                    }
                }
            }
        }
        let expected = expected_objects
            .into_iter()
            .flat_map(|(local_id, _, properties)| {
                properties
                    .into_iter()
                    .map(move |property_key| (local_id, property_key))
            })
            .collect::<Vec<_>>();
        assert!(!actual.is_empty());
        assert_eq!(actual, expected);

        reset.apply(&mut artboard);
        let empty = AnimationResetFactory::from_animation_indices(&artboard, &[], false);
        assert!(
            empty.storage.entries.is_empty(),
            "the factory must return an owned empty reset, not null"
        );
    }

    #[test]
    fn animation_reset_color_uses_cpp_signed_float_round_trip() {
        assert_eq!(
            AnimationResetColorValue::from_color(0x011d_1d1d).replay(),
            0x011d_1d1c,
            "pinned animation_reset_factory.cpp:126-168 stores color int bits as float"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0xff1d_1d1d).replay(),
            0xff1d_1d1d,
            "negative signed colors also round-trip through the C++ float representation"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0x7fff_ffff),
            AnimationResetColorValue::SaturatingFloatToInt(2_147_483_648.0),
            "2^31 cannot be converted back to C++ int with defined behavior"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0x7fff_ffff).replay(),
            0x7fff_ffff,
            "project divergence D2 saturates the otherwise-undefined conversion"
        );
    }
}
