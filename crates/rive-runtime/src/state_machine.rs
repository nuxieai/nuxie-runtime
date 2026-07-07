use crate::ArtboardInstance;
use crate::animation::{
    AnimationLoop, LinearAnimationInstance, RuntimeInterpolator, RuntimeLinearAnimation,
};
use crate::data_bind_graph::data_bind_flags_apply_target_to_source;
use crate::properties::{artboard_index_for_graph, property_key_for_name};
use rive_binary::{RuntimeFile, RuntimeObject};
use rive_graph::{ArtboardGraph, ParametricPathNode};
use std::collections::BTreeMap;

mod bindables;
mod instance;
mod transition_conditions;
pub(crate) use bindables::{
    RuntimeBindableArtboard, RuntimeBindableAsset, RuntimeBindableBoolean, RuntimeBindableColor,
    RuntimeBindableEnum, RuntimeBindableInteger, RuntimeBindableList, RuntimeBindableNumber,
    RuntimeBindableNumberDefaultViewModelSource, RuntimeBindableString, RuntimeBindableTrigger,
    RuntimeBindableTriggerSource, RuntimeBindableViewModel, RuntimeViewModelTrigger,
    StateMachineBindableArtboardInstance, StateMachineBindableAssetInstance,
    StateMachineBindableBooleanInstance, StateMachineBindableColorInstance,
    StateMachineBindableEnumInstance, StateMachineBindableIntegerInstance,
    StateMachineBindableListInstance, StateMachineBindableNumberInstance,
    StateMachineBindableStringInstance, StateMachineBindableTriggerInstance,
    StateMachineBindableViewModelInstance, bindable_artboard_value, bindable_asset_value,
    bindable_boolean_value, bindable_color_value, bindable_enum_value, bindable_integer_value,
    bindable_number_value, bindable_string_value, bindable_trigger_source_global_id,
    bindable_trigger_value, bindable_view_model_value, runtime_bindable_artboards,
    runtime_bindable_assets, runtime_bindable_booleans, runtime_bindable_colors,
    runtime_bindable_enums, runtime_bindable_integers, runtime_bindable_lists,
    runtime_bindable_numbers, runtime_bindable_strings, runtime_bindable_triggers,
    runtime_bindable_view_models, runtime_default_view_model_triggers,
    runtime_number_default_view_model_source_for_instance,
};
pub use instance::StateMachineInstance;
use transition_conditions::RuntimeTransitionCondition;

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

impl RuntimeStateMachineInput {
    pub(crate) fn new_bool(global_id: u32, name: Option<String>, value: bool) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Bool,
            value: StateMachineInputValue::Bool(value),
        }
    }

    pub(crate) fn new_number(global_id: u32, name: Option<String>, value: f32) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Number,
            value: StateMachineInputValue::Number(value),
        }
    }

    pub(crate) fn new_trigger(global_id: u32, name: Option<String>) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Trigger,
            value: StateMachineInputValue::Trigger {
                fired: false,
                used_layers: Vec::new(),
            },
        }
    }
}

pub(crate) fn runtime_state_machine_input(
    object: &RuntimeObject,
) -> Option<RuntimeStateMachineInput> {
    let name = object.string_property("name").map(ToOwned::to_owned);
    match object.type_name {
        "StateMachineBool" => Some(RuntimeStateMachineInput::new_bool(
            object.id,
            name,
            object.bool_property("value").unwrap_or(false),
        )),
        "StateMachineNumber" => Some(RuntimeStateMachineInput::new_number(
            object.id,
            name,
            object.double_property("value").unwrap_or(0.0),
        )),
        "StateMachineTrigger" => Some(RuntimeStateMachineInput::new_trigger(object.id, name)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine {
    pub global_id: u32,
    pub name: Option<String>,
    pub inputs: Vec<RuntimeStateMachineInput>,
    pub(crate) listeners: Vec<RuntimeStateMachineListener>,
    pub layers: Vec<RuntimeStateMachineLayer>,
    pub(crate) bindable_numbers: Vec<RuntimeBindableNumber>,
    pub(crate) bindable_integers: Vec<RuntimeBindableInteger>,
    pub(crate) bindable_colors: Vec<RuntimeBindableColor>,
    pub(crate) bindable_strings: Vec<RuntimeBindableString>,
    pub(crate) bindable_enums: Vec<RuntimeBindableEnum>,
    pub(crate) bindable_assets: Vec<RuntimeBindableAsset>,
    pub(crate) bindable_artboards: Vec<RuntimeBindableArtboard>,
    pub(crate) bindable_lists: Vec<RuntimeBindableList>,
    pub(crate) bindable_triggers: Vec<RuntimeBindableTrigger>,
    pub(crate) bindable_view_models: Vec<RuntimeBindableViewModel>,
    pub(crate) bindable_booleans: Vec<RuntimeBindableBoolean>,
    pub(crate) view_model_triggers: Vec<RuntimeViewModelTrigger>,
    pub(crate) transition_duration_bindings: Vec<RuntimeTransitionDurationBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTransitionDurationBinding {
    pub(crate) transition_global_id: u32,
    pub(crate) source: RuntimeBindableNumberDefaultViewModelSource,
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineTransitionDurationInstance {
    pub(crate) transition_global_id: u32,
    value: f32,
}

impl StateMachineTransitionDurationInstance {
    pub(crate) fn new(binding: &RuntimeTransitionDurationBinding) -> Self {
        Self {
            transition_global_id: binding.transition_global_id,
            value: 0.0,
        }
    }

    pub(crate) fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub(crate) fn value(&self) -> f32 {
        self.value
    }
}

pub(crate) fn build_state_machines(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
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
            let state_machine_data_binds = state_machine.data_binds.clone();
            let bindable_numbers = runtime_bindable_numbers(file, &state_machine, default_instance);
            let bindable_integers =
                runtime_bindable_integers(file, &state_machine, default_instance);
            let bindable_colors = runtime_bindable_colors(file, &state_machine, default_instance);
            let bindable_strings = runtime_bindable_strings(file, &state_machine, default_instance);
            let bindable_enums = runtime_bindable_enums(file, &state_machine, default_instance);
            let bindable_assets = runtime_bindable_assets(file, &state_machine, default_instance);
            let bindable_artboards =
                runtime_bindable_artboards(file, &state_machine, default_instance);
            let bindable_lists = runtime_bindable_lists(file, &state_machine, default_instance);
            let bindable_triggers = runtime_bindable_triggers(file, &state_machine, default_instance);
            let bindable_view_models =
                runtime_bindable_view_models(file, &state_machine, default_instance);
            let bindable_booleans = runtime_bindable_booleans(file, &state_machine, default_instance);
            let view_model_triggers =
                runtime_default_view_model_triggers(file, default_view_model_index);
            let transition_duration_bindings =
                runtime_transition_duration_bindings(file, &state_machine, artboard_index);
            RuntimeStateMachine {
                global_id: state_machine.object.id,
                name: state_machine
                    .object
                    .string_property("name")
                    .map(ToOwned::to_owned),
                inputs: state_machine
                    .inputs
                    .iter()
                    .copied()
                    .filter_map(runtime_state_machine_input)
                    .collect(),
                listeners: state_machine
                    .listeners
                    .iter()
                    .filter_map(|listener| {
                        runtime_state_machine_listener(file, graph, &state_machine_data_binds, listener)
                    })
                    .collect(),
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_lists,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                view_model_triggers,
                transition_duration_bindings,
                layers: state_machine
                    .layers
                    .into_iter()
                    .map(|layer| {
                        let states = layer
                            .states
                            .into_iter()
                            .map(|state| {
                                let animation_index = state.animation.and_then(|animation| {
                                    animation_index_by_global.get(&animation.id).copied()
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
                                    animation_index,
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
                                        .filter_map(|action| {
                                            RuntimeStateMachineFireAction::from_imported(
                                                file, action,
                                            )
                                        })
                                        .collect(),
                                    listener_actions: state
                                        .listener_actions
                                        .iter()
                                        .filter_map(|action| {
                                            RuntimeScheduledListenerAction::from_imported(
                                                file,
                                                &state_machine_data_binds,
                                                action,
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
                                                    .unwrap_or(1),
                                                condition_count: transition.conditions.len(),
                                                conditions: transition
                                                    .conditions
                                                    .iter()
                                                    .filter_map(|condition| {
                                                        RuntimeTransitionCondition::from_object(
                                                            file, graph, condition,
                                                        )
                                                    })
                                                    .collect(),
                                                fire_actions: transition
                                                    .fire_actions
                                                    .iter()
                                                    .filter_map(|action| {
                                                        RuntimeStateMachineFireAction::from_imported(
                                                            file, action,
                                                        )
                                                    })
                                                    .collect(),
                                                listener_actions: transition
                                                    .listener_actions
                                                    .iter()
                                                    .filter_map(|action| {
                                                        RuntimeScheduledListenerAction::from_imported(
                                                            file,
                                                            &state_machine_data_binds,
                                                            action,
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
                        let entry_state_index = states
                            .iter()
                            .position(|state| state.type_name == Some("EntryState"));
                        let any_state_index = states
                            .iter()
                            .position(|state| state.type_name == Some("AnyState"));
                        RuntimeStateMachineLayer {
                            global_id: layer.object.id,
                            name: layer.object.string_property("name").map(ToOwned::to_owned),
                            states,
                            entry_state_index,
                            any_state_index,
                        }
                    })
                    .collect(),
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

fn runtime_transition_duration_bindings(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
    artboard_index: usize,
) -> Vec<RuntimeTransitionDurationBinding> {
    let Some(default_instance) = state_machine_default_view_model_index(file, artboard_index)
        .and_then(|view_model_index| file.view_model_default_instance(view_model_index))
    else {
        return Vec::new();
    };
    let mut bindings = BTreeMap::<u32, RuntimeTransitionDurationBinding>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) =
            state_machine_transition_target_for_data_bind(file, state_machine, data_bind)
        else {
            continue;
        };
        if target.type_name != "StateTransition" {
            continue;
        }
        let Some(source) = runtime_number_default_view_model_source_for_instance(
            file,
            data_bind_index,
            data_bind,
            "StateTransition",
            "duration",
            default_instance.object,
        ) else {
            continue;
        };
        bindings.insert(
            target.id,
            RuntimeTransitionDurationBinding {
                transition_global_id: target.id,
                source,
            },
        );
    }
    bindings.into_values().collect()
}

fn state_machine_transition_target_for_data_bind<'a>(
    file: &'a RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'a>,
    data_bind: &RuntimeObject,
) -> Option<&'a RuntimeObject> {
    if let Some(target) = file.data_bind_target_for_object(data_bind) {
        return Some(target);
    }

    state_machine
        .layers
        .iter()
        .flat_map(|layer| &layer.states)
        .flat_map(|state| &state.transitions)
        .map(|transition| transition.object)
        .filter(|transition| transition.id < data_bind.id)
        .max_by_key(|transition| transition.id)
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineListener {
    pub(crate) target_local_id: usize,
    pub(crate) listener_types: Vec<RuntimeListenerType>,
    pub(crate) event_local_indices: Vec<usize>,
    pub(crate) hit_paths: Vec<RuntimeListenerHitPath>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
}

impl RuntimeStateMachineListener {
    pub(crate) fn has_listener(&self, listener_type: RuntimeListenerType) -> bool {
        self.listener_types.contains(&listener_type)
    }

    pub(crate) fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        if artboard
            .component(self.target_local_id)
            .is_none_or(|component| component.is_collapsed())
        {
            return false;
        }

        self.hit_paths
            .iter()
            .any(|path| path.hit_test(artboard, x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeListenerType {
    Enter,
    Exit,
    Down,
    Up,
    Move,
    Event,
    Click,
    ComponentProvided,
    TextInput,
    DragStart,
    DragEnd,
    ViewModel,
    Drag,
    Focus,
    Blur,
    Keyboard,
    SemanticAction,
    Gamepad,
}

impl RuntimeListenerType {
    fn from_value(value: u64) -> Option<Self> {
        match value {
            0 => Some(Self::Enter),
            1 => Some(Self::Exit),
            2 => Some(Self::Down),
            3 => Some(Self::Up),
            4 => Some(Self::Move),
            5 => Some(Self::Event),
            6 => Some(Self::Click),
            7 => Some(Self::ComponentProvided),
            8 => Some(Self::TextInput),
            9 => Some(Self::DragStart),
            10 => Some(Self::DragEnd),
            11 => Some(Self::ViewModel),
            12 => Some(Self::Drag),
            13 => Some(Self::Focus),
            14 => Some(Self::Blur),
            15 => Some(Self::Keyboard),
            16 => Some(Self::SemanticAction),
            17 => Some(Self::Gamepad),
            _ => None,
        }
    }

    pub(crate) fn is_pointer_hit(self) -> bool {
        matches!(
            self,
            Self::Enter
                | Self::Exit
                | Self::Down
                | Self::Up
                | Self::Move
                | Self::Click
                | Self::DragStart
                | Self::DragEnd
                | Self::Drag
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerHitPath {
    local_id: usize,
    kind: RuntimeListenerHitPathKind,
}

impl RuntimeListenerHitPath {
    fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        if artboard
            .component(self.local_id)
            .is_none_or(|component| component.is_collapsed())
        {
            return false;
        }

        match self.kind {
            RuntimeListenerHitPathKind::Rectangle => self.hit_test_rectangle(artboard, x, y),
        }
    }

    fn hit_test_rectangle(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        let Some(component) = artboard.component(self.local_id) else {
            return false;
        };
        let local = component.transform.world_transform.invert_or_identity();
        let (local_x, local_y) = local.transform_point(x, y);
        let width = artboard_double_property(artboard, self.local_id, "Rectangle", "width", 0.0);
        let height = artboard_double_property(artboard, self.local_id, "Rectangle", "height", 0.0);
        let origin_x =
            artboard_double_property(artboard, self.local_id, "Rectangle", "originX", 0.5);
        let origin_y =
            artboard_double_property(artboard, self.local_id, "Rectangle", "originY", 0.5);
        let left = -width * origin_x;
        let top = -height * origin_y;
        local_x >= left && local_x <= left + width && local_y >= top && local_y <= top + height
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeListenerHitPathKind {
    Rectangle,
}

fn runtime_state_machine_listener(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    state_machine_data_binds: &[&RuntimeObject],
    listener: &rive_binary::RuntimeStateMachineListener<'_>,
) -> Option<RuntimeStateMachineListener> {
    // Mirrors C++ StateMachineListener import/action wiring and the first
    // simple-shape branch of src/animation/state_machine_instance.cpp.
    let target_local_id = usize::try_from(listener.object.uint_property("targetId")?).ok()?;
    let listener_types = runtime_listener_types(listener)
        .into_iter()
        .filter(|listener_type| {
            listener_type.is_pointer_hit() || *listener_type == RuntimeListenerType::Event
        })
        .collect::<Vec<_>>();
    if listener_types.is_empty() {
        return None;
    }

    let hit_paths = if listener_types
        .iter()
        .any(|listener_type| listener_type.is_pointer_hit())
    {
        let hit_paths = runtime_listener_hit_paths(graph, target_local_id);
        if hit_paths.is_empty() {
            return None;
        }
        hit_paths
    } else {
        Vec::new()
    };
    let event_local_indices = runtime_listener_event_local_indices(listener);

    Some(RuntimeStateMachineListener {
        target_local_id,
        listener_types,
        event_local_indices,
        hit_paths,
        listener_actions: listener
            .actions
            .iter()
            .filter_map(|action| {
                RuntimeScheduledListenerAction::from_imported(
                    file,
                    state_machine_data_binds,
                    action,
                )
            })
            .collect(),
    })
}

fn runtime_listener_types(
    listener: &rive_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerType> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        return RuntimeListenerType::from_value(
            listener
                .object
                .uint_property("listenerTypeValue")
                .unwrap_or(0),
        )
        .into_iter()
        .collect();
    }

    listener
        .listener_input_types
        .iter()
        .map(|input_type| input_type.uint_property("listenerTypeValue").unwrap_or(0))
        .filter_map(RuntimeListenerType::from_value)
        .collect()
}

fn runtime_listener_event_local_indices(
    listener: &rive_binary::RuntimeStateMachineListener<'_>,
) -> Vec<usize> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        let listener_type = listener
            .object
            .uint_property("listenerTypeValue")
            .and_then(RuntimeListenerType::from_value);
        if listener_type != Some(RuntimeListenerType::Event) {
            return Vec::new();
        }
        return listener
            .object
            .uint_property("eventId")
            .and_then(|event_id| usize::try_from(event_id).ok())
            .into_iter()
            .collect();
    }

    listener
        .listener_input_types
        .iter()
        .filter(|input_type| {
            input_type
                .uint_property("listenerTypeValue")
                .and_then(RuntimeListenerType::from_value)
                == Some(RuntimeListenerType::Event)
        })
        .filter_map(|input_type| {
            input_type
                .uint_property("eventId")
                .and_then(|event_id| usize::try_from(event_id).ok())
        })
        .collect()
}

fn runtime_listener_hit_paths(
    graph: &ArtboardGraph,
    target_local_id: usize,
) -> Vec<RuntimeListenerHitPath> {
    let Some(target) = graph
        .components
        .iter()
        .find(|component| component.local_id == target_local_id)
    else {
        return Vec::new();
    };
    if target.type_name != "Shape" {
        // TODO(golden): port C++ StateMachineInstance::addToHitLookup for
        // containers, layout proxies, text runs, and component-provided groups.
        return Vec::new();
    }

    let Some(composer) = graph
        .path_composers
        .iter()
        .find(|composer| composer.shape_local == target_local_id)
    else {
        return Vec::new();
    };

    composer
        .path_locals
        .iter()
        .filter_map(|path_local| {
            let path = graph
                .paths
                .iter()
                .find(|path| path.local_id == *path_local)?;
            match path.parametric {
                Some(ParametricPathNode::Rectangle { .. }) => Some(RuntimeListenerHitPath {
                    local_id: *path_local,
                    kind: RuntimeListenerHitPathKind::Rectangle,
                }),
                _ => None,
            }
        })
        .collect()
}

fn artboard_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone)]
enum StateMachineInputValue {
    Bool(bool),
    Number(f32),
    Trigger {
        fired: bool,
        used_layers: Vec<usize>,
    },
}

// Mirrors the runtime event report surface threaded through state-machine advancement.
#[derive(Debug, Clone)]
pub struct StateMachineReportedEvent {
    pub(crate) event_local_index: usize,
    pub(crate) event_core_type: u32,
    pub(crate) name: Option<String>,
    pub(crate) seconds_delay: f32,
}

impl StateMachineReportedEvent {
    pub fn event_local_index(&self) -> usize {
        self.event_local_index
    }

    pub fn event_core_type(&self) -> u32 {
        self.event_core_type
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer {
    pub global_id: u32,
    pub name: Option<String>,
    pub states: Vec<RuntimeLayerState>,
    pub(crate) entry_state_index: Option<usize>,
    pub(crate) any_state_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLayerState {
    pub global_id: Option<u32>,
    pub type_name: Option<&'static str>,
    pub(crate) animation_index: Option<usize>,
    pub(crate) blend_state_1d: Option<RuntimeBlendState1D>,
    pub(crate) blend_state_direct: Option<RuntimeBlendStateDirect>,
    pub(crate) speed: f32,
    pub(crate) flags: u64,
    pub(crate) fire_actions: Vec<RuntimeStateMachineFireAction>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
    pub(crate) transitions: Vec<RuntimeStateTransition>,
}

impl RuntimeLayerState {
    const RANDOM: u64 = 1 << 0;
    const RESET: u64 = 1 << 1;

    fn uses_random_transition_selection(&self) -> bool {
        self.flags & Self::RANDOM == Self::RANDOM
    }

    fn resets_blend_values(&self) -> bool {
        self.flags & Self::RESET == Self::RESET
    }

    fn perform_fire_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        perform_state_machine_fire_actions(
            &self.fire_actions,
            occurrence,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
        );
    }

    fn perform_listener_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) -> bool {
        perform_scheduled_listener_actions(
            &self.listener_actions,
            occurrence,
            inputs,
            reported_events,
            pending_view_model_actions,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateTransition {
    pub(crate) global_id: u32,
    pub(crate) state_to_index: Option<usize>,
    pub(crate) exit_blend_animation_index: Option<usize>,
    pub(crate) duration: u64,
    pub(crate) exit_time: u64,
    pub(crate) flags: u64,
    pub(crate) random_weight: u64,
    pub(crate) condition_count: usize,
    conditions: Vec<RuntimeTransitionCondition>,
    pub(crate) fire_actions: Vec<RuntimeStateMachineFireAction>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
    pub(crate) interpolator: Option<RuntimeTransitionInterpolator>,
    pub(crate) has_unsupported_interpolator: bool,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeTransitionAnimationRef<'a> {
    instance: &'a LinearAnimationInstance,
    animation: &'a RuntimeLinearAnimation,
}

impl RuntimeStateTransition {
    const DISABLED: u64 = 1 << 0;
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
    const PAUSE_ON_EXIT: u64 = 1 << 4;
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    fn is_simple_supported(&self) -> bool {
        self.state_to_index.is_some()
            && self.condition_count == self.conditions.len()
            && !self.has_unsupported_interpolator
            && self.flags & Self::DISABLED == 0
    }

    fn allow(
        &self,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        view_model_triggers: &[StateMachineViewModelTriggerInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
        layer_index: usize,
        animation_from: Option<RuntimeTransitionAnimationRef<'_>>,
    ) -> TransitionAllowance {
        if !self.conditions.iter().all(|condition| {
            condition.evaluate(
                artboard,
                inputs,
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                view_model_triggers,
                data_context_present,
                data_context_view_model_bound,
                layer_index,
            )
        }) {
            return TransitionAllowance::No;
        }

        if self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
            && let Some(animation_from) = animation_from
        {
            let mut exit_time = self.exit_time_seconds(Some(animation_from.animation), false);
            let duration = animation_from.animation.duration_seconds();
            if exit_time <= duration
                && AnimationLoop::from_loop_value(animation_from.animation.loop_value)
                    != AnimationLoop::OneShot
                && duration != 0.0
            {
                exit_time +=
                    (animation_from.instance.last_total_time / duration).floor() * duration;
            }
            if animation_from.instance.total_time < exit_time {
                return TransitionAllowance::WaitingForExit;
            }
        }

        TransitionAllowance::Yes
    }

    fn has_exit_time(&self) -> bool {
        self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
    }

    fn pause_on_exit(&self) -> bool {
        self.flags & Self::PAUSE_ON_EXIT == Self::PAUSE_ON_EXIT
    }

    fn enable_early_exit(&self) -> bool {
        self.flags & Self::ENABLE_EARLY_EXIT == Self::ENABLE_EARLY_EXIT
    }

    fn perform_fire_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        perform_state_machine_fire_actions(
            &self.fire_actions,
            occurrence,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
        );
    }

    fn perform_listener_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) -> bool {
        perform_scheduled_listener_actions(
            &self.listener_actions,
            occurrence,
            inputs,
            reported_events,
            pending_view_model_actions,
        )
    }

    fn transition_duration_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        duration_override: Option<f32>,
    ) -> f32 {
        let duration = duration_override
            .map(|value| if value < 0.0 { 0 } else { value.round() as u64 })
            .unwrap_or(self.duration);
        if duration == 0 {
            return 0.0;
        }
        if self.flags & Self::DURATION_IS_PERCENTAGE == Self::DURATION_IS_PERCENTAGE {
            return animation_from
                .map(|animation| duration as f32 / 100.0 * animation.duration_seconds())
                .unwrap_or(0.0);
        }
        duration as f32 / 1000.0
    }

    fn exit_time_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        absolute: bool,
    ) -> f32 {
        if self.flags & Self::EXIT_TIME_IS_PERCENTAGE == Self::EXIT_TIME_IS_PERCENTAGE {
            return animation_from
                .map(|animation| {
                    let start = if absolute {
                        animation.start_seconds()
                    } else {
                        0.0
                    };
                    start + self.exit_time as f32 / 100.0 * animation.duration_seconds()
                })
                .unwrap_or(0.0);
        }
        self.exit_time as f32 / 1000.0
    }

    fn use_inputs(
        &self,
        inputs: &mut [StateMachineInputInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        layer_index: usize,
    ) {
        for condition in &self.conditions {
            condition.use_input(inputs, bindable_triggers, view_model_triggers, layer_index);
        }
    }
}

fn transition_duration_value(
    transition_durations: &[StateMachineTransitionDurationInstance],
    transition_global_id: u32,
) -> Option<f32> {
    transition_durations
        .iter()
        .find(|duration| duration.transition_global_id == transition_global_id)
        .map(StateMachineTransitionDurationInstance::value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionAllowance {
    No,
    WaitingForExit,
    Yes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateMachineFireOccurrence {
    AtStart,
    AtEnd,
}

impl StateMachineFireOccurrence {
    pub(crate) fn value(self) -> u64 {
        match self {
            Self::AtStart => 0,
            Self::AtEnd => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeStateMachineFireAction {
    Event {
        occurs_value: u64,
        event: StateMachineReportedEvent,
    },
    Trigger {
        occurs_value: u64,
        target_global_id: Option<u32>,
    },
}

impl RuntimeStateMachineFireAction {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        action: &rive_binary::RuntimeStateMachineFireAction<'_>,
    ) -> Option<Self> {
        let occurs_value = action.object.uint_property("occursValue").unwrap_or(0);
        match action.object.type_name {
            "StateMachineFireEvent" => {
                let event = action.event?;
                Some(Self::Event {
                    occurs_value,
                    event: StateMachineReportedEvent {
                        event_local_index: action.event_local_index?,
                        event_core_type: u32::from(event.type_key),
                        name: event.string_property("name").map(ToOwned::to_owned),
                        seconds_delay: 0.0,
                    },
                })
            }
            "StateMachineFireTrigger" => Some(Self::Trigger {
                occurs_value,
                target_global_id: runtime_fire_trigger_target_global(file, action.object),
            }),
            _ => None,
        }
    }
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
        state: &rive_binary::RuntimeLayerState<'_>,
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
                global_id: file.latest_bindable_property_for_object(object)?.id as u32,
            },
            _ => return None,
        };
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                let animation_index = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())?;
                Some(RuntimeBlendAnimation1D {
                    animation_index,
                    value: animation.object.double_property("value").unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();
        (!animations.is_empty()).then_some(Self { source, animations })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeBlendState1DSource {
    Input { input_index: Option<usize> },
    BindableProperty { global_id: u32 },
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimation1D {
    pub(crate) animation_index: usize,
    pub(crate) value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendStateDirect {
    pub(crate) animations: Vec<RuntimeBlendAnimationDirect>,
}

impl RuntimeBlendStateDirect {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state: &rive_binary::RuntimeLayerState<'_>,
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
                let animation_index = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())?;
                Some(RuntimeBlendAnimationDirect {
                    animation_index,
                    source: RuntimeDirectBlendSource::from_object(file, animation.object)?,
                })
            })
            .collect::<Vec<_>>();
        (!animations.is_empty()).then_some(Self { animations })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimationDirect {
    pub(crate) animation_index: usize,
    pub(crate) source: RuntimeDirectBlendSource,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeDirectBlendSource {
    Input { input_index: usize },
    MixValue { value: f32 },
    BindableProperty { global_id: u32 },
}

impl RuntimeDirectBlendSource {
    fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Option<Self> {
        match object.uint_property("blendSource").unwrap_or(0) {
            0 => Some(Self::Input {
                input_index: usize::try_from(object.uint_property("inputId")?).ok()?,
            }),
            1 => Some(Self::MixValue {
                value: object.double_property("mixValue").unwrap_or(100.0),
            }),
            2 => Some(Self::BindableProperty {
                global_id: file.latest_bindable_property_for_object(object)?.id as u32,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BlendState1DInstance {
    source: RuntimeBlendState1DSource,
    animations: Vec<BlendAnimation1DInstance>,
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
            .filter_map(|animation| {
                let linear_animation = artboard.linear_animation(animation.animation_index)?;
                Some(BlendAnimation1DInstance {
                    value: animation.value,
                    animation: LinearAnimationInstance::new(
                        animation.animation_index,
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
                .map(|animation| animation.animation_index)
                .collect::<Vec<_>>();
            AnimationReset::from_animation_indices(artboard, &animation_indices, true)
        } else {
            None
        };

        Self {
            source: blend_state.source.clone(),
            animations,
            animation_reset,
        }
    }

    pub(crate) fn advance(
        &mut self,
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

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
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

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        if self.animations.is_empty() {
            return;
        }

        let value = match self.source {
            RuntimeBlendState1DSource::Input { input_index } => input_index
                .and_then(|input_index| inputs.get(input_index))
                .and_then(StateMachineInputInstance::number_value)
                .unwrap_or(0.0),
            RuntimeBlendState1DSource::BindableProperty { global_id } => {
                bindable_number_value(bindable_numbers, global_id).unwrap_or(0.0)
            }
        };

        let animation_count = self.animations.len();
        let to_index = self.animation_index(value);
        let from_index = to_index.checked_sub(1);
        let to_value = self
            .animations
            .get(to_index)
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let from_value = from_index
            .and_then(|index| self.animations.get(index))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let (mix, mix_from) =
            if to_index >= animation_count || from_index.is_none() || to_value == from_value {
                (1.0, 1.0)
            } else {
                let mix = (value - from_value) / (to_value - from_value);
                (mix, 1.0 - mix)
            };

        for animation in &mut self.animations {
            if to_index < animation_count && animation.value == to_value {
                animation.mix = mix;
            } else if from_index.is_some() && animation.value == from_value {
                animation.mix = mix_from;
            } else {
                animation.mix = 0.0;
            }
        }
    }

    fn animation_index(&self, value: f32) -> usize {
        let mut index = 0_usize;
        let mut start = 0_isize;
        let mut end = self.animations.len() as isize - 1;

        while start <= end {
            let mid = (start + end) >> 1;
            let closest_value = self.animations[mid as usize].value;
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
            .get(index)
            .map(|animation| &animation.animation)
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
            changed |=
                artboard.apply_linear_animation_instance(&animation.animation, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimation1DInstance {
    value: f32,
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
            .filter_map(|animation| {
                let linear_animation = artboard.linear_animation(animation.animation_index)?;
                Some(BlendAnimationDirectInstance {
                    source: animation.source.clone(),
                    animation: LinearAnimationInstance::new(
                        animation.animation_index,
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

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
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

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        for animation in &mut self.animations {
            let value = match animation.source {
                RuntimeDirectBlendSource::Input { input_index } => inputs
                    .get(input_index)
                    .and_then(StateMachineInputInstance::number_value)
                    .unwrap_or(0.0),
                RuntimeDirectBlendSource::MixValue { value } => value,
                RuntimeDirectBlendSource::BindableProperty { global_id } => {
                    bindable_number_value(bindable_numbers, global_id).unwrap_or(0.0)
                }
            };
            animation.mix = (value / 100.0).clamp(0.0, 1.0);
        }
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .get(index)
            .map(|animation| &animation.animation)
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |=
                artboard.apply_linear_animation_instance(&animation.animation, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimationDirectInstance {
    source: RuntimeDirectBlendSource,
    animation: LinearAnimationInstance,
    mix: f32,
}

pub(crate) fn perform_state_machine_fire_actions(
    fire_actions: &[RuntimeStateMachineFireAction],
    occurrence: StateMachineFireOccurrence,
    data_context_view_model_bound: bool,
    view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
    reported_events: &mut Vec<StateMachineReportedEvent>,
) {
    for action in fire_actions {
        match action {
            RuntimeStateMachineFireAction::Event {
                occurs_value,
                event,
            } if *occurs_value == occurrence.value() => {
                reported_events.push(event.clone());
            }
            RuntimeStateMachineFireAction::Trigger {
                occurs_value,
                target_global_id,
            } if *occurs_value == occurrence.value() && data_context_view_model_bound => {
                if let Some(target_global_id) = target_global_id {
                    if let Some(trigger) = view_model_triggers
                        .iter_mut()
                        .find(|trigger| trigger.global_id() == *target_global_id)
                    {
                        trigger.increment();
                    }
                }
            }
            _ => {}
        }
    }
}

fn runtime_fire_trigger_target_global(file: &RuntimeFile, object: &RuntimeObject) -> Option<u32> {
    let data_bind_path = file.data_bind_path_for_referencer_object(object)?;
    let is_relative = data_bind_path
        .object
        .and_then(|path_object| path_object.bool_property("isRelative"))
        .or_else(|| object.bool_property("isDataBindPathRelative"))
        .unwrap_or(false);
    if is_relative {
        return None;
    }
    let default_instance = file.view_model_default_instance(0)?;
    let target = file.data_context_view_model_property_for_instance(
        default_instance.object,
        &data_bind_path.resolved_path_ids,
    )?;
    file.view_model_instance_trigger_count_for_object(target)?;
    Some(target.id)
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeScheduledListenerAction {
    FireEvent {
        flags: u64,
        event: StateMachineReportedEvent,
    },
    BoolChange {
        flags: u64,
        input_index: usize,
        value: u64,
    },
    NumberChange {
        flags: u64,
        input_index: usize,
        value: f32,
    },
    TriggerChange {
        flags: u64,
        input_index: usize,
    },
    ViewModelChange {
        flags: u64,
        data_bind_index: usize,
        value: RuntimeListenerViewModelChangeValue,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeListenerViewModelChangeValue {
    Number(f32),
    Integer(u64),
    Color(u32),
    String(Vec<u8>),
    Enum(u64),
    Asset(u64),
    Artboard(u64),
    Trigger(u64),
    Boolean(bool),
}

impl RuntimeScheduledListenerAction {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state_machine_data_binds: &[&RuntimeObject],
        action: &rive_binary::RuntimeListenerAction<'_>,
    ) -> Option<Self> {
        let flags = action.object.uint_property("flags").unwrap_or(0);
        match action.object.type_name {
            "ListenerFireEvent" => {
                let event = action.event?;
                Some(Self::FireEvent {
                    flags,
                    event: StateMachineReportedEvent {
                        event_local_index: action.event_local_index?,
                        event_core_type: u32::from(event.type_key),
                        name: event.string_property("name").map(ToOwned::to_owned),
                        seconds_delay: 0.0,
                    },
                })
            }
            "ListenerBoolChange" => Some(Self::BoolChange {
                flags,
                input_index: listener_action_input_index(action)?,
                value: action.object.uint_property("value").unwrap_or(1),
            }),
            "ListenerNumberChange" => Some(Self::NumberChange {
                flags,
                input_index: listener_action_input_index(action)?,
                value: action.object.double_property("value").unwrap_or(0.0),
            }),
            "ListenerTriggerChange" => Some(Self::TriggerChange {
                flags,
                input_index: listener_action_input_index(action)?,
            }),
            "ListenerViewModelChange" => runtime_listener_view_model_change_action(
                file,
                state_machine_data_binds,
                action,
                flags,
            ),
            _ => None,
        }
    }
}

fn runtime_listener_view_model_change_action(
    file: &RuntimeFile,
    state_machine_data_binds: &[&RuntimeObject],
    action: &rive_binary::RuntimeListenerAction<'_>,
    flags: u64,
) -> Option<RuntimeScheduledListenerAction> {
    // Mirrors src/animation/listener_viewmodel_change.cpp import-time
    // BindableProperty lookup, using object ids/data-bind indices instead of
    // the C++ per-instance pointer maps.
    let bindable_property_id = action
        .object
        .uint_property("bindablePropertyId")
        .and_then(|id| u32::try_from(id).ok())
        .or_else(|| latest_bindable_property_id_before(file, action.object.id))?;

    state_machine_data_binds
        .iter()
        .enumerate()
        .find_map(|(data_bind_index, data_bind)| {
            let target = file.data_bind_target_for_object(data_bind)?;
            if target.id != bindable_property_id {
                return None;
            }
            let data_bind_flags = data_bind.uint_property("flags").unwrap_or(0);
            if !data_bind_flags_apply_target_to_source(data_bind_flags) {
                return None;
            }
            let value = runtime_listener_view_model_change_value(target)?;
            Some(RuntimeScheduledListenerAction::ViewModelChange {
                flags,
                data_bind_index,
                value,
            })
        })
}

fn latest_bindable_property_id_before(file: &RuntimeFile, object_id: u32) -> Option<u32> {
    file.objects
        .iter()
        .rev()
        .flatten()
        .find(|object| object.id < object_id && is_bindable_property(object))
        .map(|object| object.id)
}

fn is_bindable_property(object: &RuntimeObject) -> bool {
    matches!(
        object.type_name,
        "BindablePropertyNumber"
            | "BindablePropertyInteger"
            | "BindablePropertyColor"
            | "BindablePropertyString"
            | "BindablePropertyEnum"
            | "BindablePropertyAsset"
            | "BindablePropertyArtboard"
            | "BindablePropertyList"
            | "BindablePropertyTrigger"
            | "BindablePropertyViewModel"
            | "BindablePropertyBoolean"
    )
}

fn runtime_listener_view_model_change_value(
    target: &RuntimeObject,
) -> Option<RuntimeListenerViewModelChangeValue> {
    match target.type_name {
        "BindablePropertyNumber" => Some(RuntimeListenerViewModelChangeValue::Number(
            target.double_property("propertyValue").unwrap_or(0.0),
        )),
        "BindablePropertyInteger" => Some(RuntimeListenerViewModelChangeValue::Integer(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyColor" => Some(RuntimeListenerViewModelChangeValue::Color(
            target.color_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyString" => Some(RuntimeListenerViewModelChangeValue::String(
            target
                .string_property_bytes("propertyValue")
                .unwrap_or_default()
                .to_vec(),
        )),
        "BindablePropertyEnum" => Some(RuntimeListenerViewModelChangeValue::Enum(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyAsset" => Some(RuntimeListenerViewModelChangeValue::Asset(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyArtboard" => Some(RuntimeListenerViewModelChangeValue::Artboard(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyList" => None,
        "BindablePropertyTrigger" => Some(RuntimeListenerViewModelChangeValue::Trigger(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyBoolean" => Some(RuntimeListenerViewModelChangeValue::Boolean(
            target.bool_property("propertyValue").unwrap_or(false),
        )),
        _ => None,
    }
}

fn listener_action_input_index(action: &rive_binary::RuntimeListenerAction<'_>) -> Option<usize> {
    if action
        .object
        .uint_property("nestedInputId")
        .is_some_and(|nested_input_id| nested_input_id != u64::from(u32::MAX))
    {
        return None;
    }
    usize::try_from(action.object.uint_property("inputId")?).ok()
}

pub(crate) fn perform_scheduled_listener_actions(
    listener_actions: &[RuntimeScheduledListenerAction],
    occurrence: StateMachineFireOccurrence,
    inputs: &mut [StateMachineInputInstance],
    reported_events: &mut Vec<StateMachineReportedEvent>,
    pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
) -> bool {
    let mut changed_input = false;
    for action in listener_actions {
        let flags = match action {
            RuntimeScheduledListenerAction::FireEvent { flags, .. }
            | RuntimeScheduledListenerAction::BoolChange { flags, .. }
            | RuntimeScheduledListenerAction::NumberChange { flags, .. }
            | RuntimeScheduledListenerAction::TriggerChange { flags, .. }
            | RuntimeScheduledListenerAction::ViewModelChange { flags, .. } => *flags,
        };
        if flags & 1 != occurrence.value() {
            continue;
        }
        match action {
            RuntimeScheduledListenerAction::FireEvent { event, .. } => {
                reported_events.push(event.clone());
            }
            RuntimeScheduledListenerAction::BoolChange {
                input_index, value, ..
            } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.apply_listener_bool_change(*value);
                }
            }
            RuntimeScheduledListenerAction::NumberChange {
                input_index, value, ..
            } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.set_number(*value);
                }
            }
            RuntimeScheduledListenerAction::TriggerChange { input_index, .. } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.fire_trigger();
                }
            }
            RuntimeScheduledListenerAction::ViewModelChange { .. } => {
                pending_view_model_actions.push(action.clone());
            }
        }
    }
    changed_input
}

#[derive(Debug, Clone)]
pub struct StateMachineInputInstance {
    index: usize,
    global_id: u32,
    name: Option<String>,
    kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

impl StateMachineInputInstance {
    pub(crate) fn new(index: usize, input: &RuntimeStateMachineInput) -> Self {
        Self {
            index,
            global_id: input.global_id,
            name: input.name.clone(),
            kind: input.kind,
            value: input.value.clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn global_id(&self) -> u32 {
        self.global_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> StateMachineInputKind {
        self.kind
    }

    pub fn bool_value(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn number_value(&self) -> Option<f32> {
        match self.value {
            StateMachineInputValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn trigger_fired(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Trigger { fired, .. } => Some(fired),
            _ => None,
        }
    }

    pub(crate) fn set_bool(&mut self, value: bool) -> bool {
        match &mut self.value {
            StateMachineInputValue::Bool(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn set_number(&mut self, value: f32) -> bool {
        match &mut self.value {
            StateMachineInputValue::Number(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn apply_listener_bool_change(&mut self, value: u64) -> bool {
        match &mut self.value {
            StateMachineInputValue::Bool(current) => {
                let next = match value {
                    0 => false,
                    1 => true,
                    _ => !*current,
                };
                if *current == next {
                    return false;
                }
                *current = next;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn fire_trigger(&mut self) -> bool {
        match &mut self.value {
            StateMachineInputValue::Trigger { fired, .. } => {
                if *fired {
                    return false;
                }
                *fired = true;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn trigger_is_fireable_for_layer(&self, layer_index: usize) -> Option<bool> {
        match &self.value {
            StateMachineInputValue::Trigger { fired, used_layers } => {
                Some(*fired && !used_layers.contains(&layer_index))
            }
            _ => None,
        }
    }

    pub(crate) fn use_trigger_in_layer(&mut self, layer_index: usize) {
        if let StateMachineInputValue::Trigger { used_layers, .. } = &mut self.value
            && !used_layers.contains(&layer_index)
        {
            used_layers.push(layer_index);
        }
    }

    pub(crate) fn advanced(&mut self) {
        if let StateMachineInputValue::Trigger { fired, used_layers } = &mut self.value {
            *fired = false;
            used_layers.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineViewModelTriggerInstance {
    global_id: u32,
    view_model_property_id: u32,
    value: u64,
    changed: bool,
    used_layers: Vec<usize>,
}

impl StateMachineViewModelTriggerInstance {
    pub(crate) fn new(global_id: u32, view_model_property_id: u32, value: u64) -> Self {
        Self {
            global_id,
            view_model_property_id,
            value,
            changed: false,
            used_layers: Vec::new(),
        }
    }

    pub(crate) fn global_id(&self) -> u32 {
        self.global_id
    }

    pub(crate) fn increment(&mut self) {
        self.value = self.value.saturating_add(1);
        self.changed = true;
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        self.changed = true;
        true
    }

    pub(crate) fn replace_value(&mut self, value: u64) {
        self.value = value;
        self.changed = false;
        self.used_layers.clear();
    }

    pub(crate) fn reset(&mut self) {
        self.value = 0;
        self.changed = false;
        self.used_layers.clear();
    }

    pub(crate) fn is_fireable_for_layer(&self, layer_index: usize) -> bool {
        self.changed && !self.used_layers.contains(&layer_index)
    }

    pub(crate) fn use_in_layer(&mut self, layer_index: usize) {
        if !self.used_layers.contains(&layer_index) {
            self.used_layers.push(layer_index);
        }
    }

    pub(crate) fn value(&self) -> u64 {
        self.value
    }

    pub(crate) fn view_model_property_id(&self) -> u32 {
        self.view_model_property_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineLayerInstance {
    current_state_index: Option<usize>,
    current_animation: Option<LinearAnimationInstance>,
    current_blend_state_1d: Option<BlendState1DInstance>,
    current_blend_state_direct: Option<BlendStateDirectInstance>,
    current_animation_keep_going: bool,
    transition_source_state_index: Option<usize>,
    transition_source_animation: Option<LinearAnimationInstance>,
    transition_source_blend_state_1d: Option<BlendState1DInstance>,
    transition_source_blend_state_direct: Option<BlendStateDirectInstance>,
    transition_duration_seconds: f32,
    transition_mix: f32,
    transition_mix_from: f32,
    transition_source_paused: bool,
    transition_interpolator: Option<RuntimeTransitionInterpolator>,
    transition_enable_early_exit: bool,
    transition_fire_actions: Vec<RuntimeStateMachineFireAction>,
    transition_listener_actions: Vec<RuntimeScheduledListenerAction>,
    transition_completed: bool,
    transition_animation_reset: Option<AnimationReset>,
    waiting_for_exit: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineLayerAdvance {
    pub(crate) changed_state: bool,
    pub(crate) keep_going: bool,
    pub(crate) pending_view_model_actions: Vec<RuntimeScheduledListenerAction>,
}

#[derive(Debug, Clone)]
struct AnimationReset {
    entries: Vec<AnimationResetEntry>,
}

#[derive(Debug, Clone)]
enum AnimationResetEntry {
    Double {
        local_id: usize,
        property_key: u16,
        value: f32,
    },
    Color {
        local_id: usize,
        property_key: u16,
        value: u32,
    },
}

impl AnimationReset {
    fn from_animation_indices(
        artboard: &ArtboardInstance,
        animation_indices: &[usize],
        use_first_as_baseline: bool,
    ) -> Option<Self> {
        let mut entries = Vec::new();
        let mut seen = Vec::new();

        for (animation_order, animation_index) in animation_indices.iter().enumerate() {
            let Some(animation) = artboard.linear_animation(*animation_index) else {
                continue;
            };
            let use_baseline = use_first_as_baseline && animation_order == 0;
            for keyed_object in &animation.keyed_objects {
                for keyed_property in &keyed_object.keyed_properties {
                    if !keyed_property.double_property && !keyed_property.color_property {
                        continue;
                    };
                    let key = (keyed_object.target_local_id, keyed_property.property_key);
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.push(key);
                    if keyed_property.double_property {
                        let value = if use_baseline {
                            keyed_property.key_frames.first().map(|frame| frame.value)
                        } else {
                            current_animation_reset_double_value(
                                artboard,
                                keyed_object.target_local_id,
                                keyed_property,
                            )
                        };
                        if let Some(value) = value {
                            entries.push(AnimationResetEntry::Double {
                                local_id: keyed_object.target_local_id,
                                property_key: keyed_property.property_key,
                                value,
                            });
                        }
                    } else if keyed_property.color_property {
                        let value = if use_baseline {
                            keyed_property
                                .color_key_frames
                                .first()
                                .map(|frame| frame.value)
                        } else {
                            Some(
                                artboard
                                    .color_property(
                                        keyed_object.target_local_id,
                                        keyed_property.property_key,
                                    )
                                    .unwrap_or(keyed_property.color_source_value),
                            )
                        };
                        if let Some(value) = value {
                            entries.push(AnimationResetEntry::Color {
                                local_id: keyed_object.target_local_id,
                                property_key: keyed_property.property_key,
                                value,
                            });
                        }
                    }
                }
            }
        }

        if entries.is_empty() {
            None
        } else {
            Some(Self { entries })
        }
    }

    fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        for entry in &self.entries {
            match entry {
                AnimationResetEntry::Double {
                    local_id,
                    property_key,
                    value,
                } => {
                    changed |= artboard.set_double_property(*local_id, *property_key, *value);
                }
                AnimationResetEntry::Color {
                    local_id,
                    property_key,
                    value,
                } => {
                    changed |= artboard.set_color_property(*local_id, *property_key, *value);
                }
            }
        }
        changed
    }
}

fn current_animation_reset_double_value(
    artboard: &ArtboardInstance,
    local_id: usize,
    keyed_property: &crate::animation::RuntimeKeyedProperty,
) -> Option<f32> {
    if let Some(property) = keyed_property.transform_property {
        artboard.transform_property(local_id, property)
    } else {
        Some(
            artboard
                .double_property(local_id, keyed_property.property_key)
                .unwrap_or(keyed_property.double_source_value),
        )
    }
}

impl StateMachineLayerInstance {
    pub(crate) fn new(
        layer: &RuntimeStateMachineLayer,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> Self {
        let mut instance = Self {
            current_state_index: layer.entry_state_index,
            current_animation: None,
            current_blend_state_1d: None,
            current_blend_state_direct: None,
            current_animation_keep_going: false,
            transition_source_state_index: None,
            transition_source_animation: None,
            transition_source_blend_state_1d: None,
            transition_source_blend_state_direct: None,
            transition_duration_seconds: 0.0,
            transition_mix: 1.0,
            transition_mix_from: 1.0,
            transition_source_paused: false,
            transition_interpolator: None,
            transition_enable_early_exit: false,
            transition_fire_actions: Vec::new(),
            transition_listener_actions: Vec::new(),
            transition_completed: false,
            transition_animation_reset: None,
            waiting_for_exit: false,
        };
        instance.refresh_current_animation(artboard, layer, inputs, bindable_numbers);
        instance
    }

    pub(crate) fn has_current_animation(&self) -> bool {
        self.current_animation.is_some()
    }

    pub(crate) fn current_animation(&self) -> Option<&LinearAnimationInstance> {
        self.current_animation.as_ref()
    }

    pub(crate) fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        transition_durations: &[StateMachineTransitionDurationInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> StateMachineLayerAdvance {
        let mut pending_view_model_actions = Vec::new();
        self.advance_current_animation(
            artboard,
            layer,
            elapsed_seconds,
            inputs,
            bindable_numbers,
            reported_events,
        );
        let input_changed = self.update_transition_mix(
            elapsed_seconds,
            inputs,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
            &mut pending_view_model_actions,
        );
        self.advance_transition_source_animation(
            artboard,
            layer,
            elapsed_seconds,
            inputs,
            bindable_numbers,
            reported_events,
        );
        self.apply_animations(artboard);

        let mut changed_state = false;
        for _ in 0..100 {
            if !self.update_state(
                artboard,
                layer,
                layer_index,
                inputs,
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                transition_durations,
                data_context_present,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
                &mut pending_view_model_actions,
            ) {
                break;
            }
            changed_state = true;
            self.apply_animations(artboard);
        }

        StateMachineLayerAdvance {
            changed_state,
            keep_going: changed_state
                || input_changed
                || !pending_view_model_actions.is_empty()
                || self.is_transitioning()
                || self.waiting_for_exit
                || self.current_animation_keep_going,
            pending_view_model_actions,
        }
    }

    fn update_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        transition_durations: &[StateMachineTransitionDurationInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) -> bool {
        if self.is_transitioning() && !self.transition_enable_early_exit {
            return false;
        }
        self.waiting_for_exit = false;
        if self.try_change_state(
            artboard,
            layer,
            layer.any_state_index,
            layer_index,
            inputs,
            bindable_numbers,
            bindable_integers,
            bindable_colors,
            bindable_strings,
            bindable_enums,
            bindable_assets,
            bindable_artboards,
            bindable_triggers,
            bindable_view_models,
            bindable_booleans,
            transition_durations,
            data_context_present,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
            pending_view_model_actions,
        ) {
            return true;
        }
        self.try_change_state(
            artboard,
            layer,
            self.current_state_index,
            layer_index,
            inputs,
            bindable_numbers,
            bindable_integers,
            bindable_colors,
            bindable_strings,
            bindable_enums,
            bindable_assets,
            bindable_artboards,
            bindable_triggers,
            bindable_view_models,
            bindable_booleans,
            transition_durations,
            data_context_present,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
            pending_view_model_actions,
        )
    }

    fn try_change_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        state_index: Option<usize>,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        transition_durations: &[StateMachineTransitionDurationInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) -> bool {
        let Some(state_index) = state_index else {
            return false;
        };
        let Some(state) = layer.states.get(state_index) else {
            return false;
        };

        if state.uses_random_transition_selection() {
            let Some((transition_index, state_to_index)) = self.find_random_transition(
                artboard,
                state,
                state_index,
                layer_index,
                inputs,
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                view_model_triggers,
                data_context_present,
                data_context_view_model_bound,
            ) else {
                return false;
            };
            let transition = &state.transitions[transition_index];
            transition.use_inputs(inputs, bindable_triggers, view_model_triggers, layer_index);
            self.change_state(
                artboard,
                layer,
                transition,
                state_to_index,
                inputs,
                bindable_numbers,
                transition_durations,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
                pending_view_model_actions,
            );
            return true;
        }

        for transition in &state.transitions {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }
            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index == Some(state_index),
            );
            match transition.allow(
                artboard,
                inputs,
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                view_model_triggers,
                data_context_present,
                data_context_view_model_bound,
                layer_index,
                animation_from,
            ) {
                TransitionAllowance::No => continue,
                TransitionAllowance::WaitingForExit => {
                    self.waiting_for_exit = true;
                    continue;
                }
                TransitionAllowance::Yes => {
                    self.waiting_for_exit = false;
                }
            }
            transition.use_inputs(inputs, bindable_triggers, view_model_triggers, layer_index);
            self.change_state(
                artboard,
                layer,
                transition,
                state_to_index,
                inputs,
                bindable_numbers,
                transition_durations,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
                pending_view_model_actions,
            );
            return true;
        }
        false
    }

    fn find_random_transition(
        &mut self,
        artboard: &ArtboardInstance,
        state: &RuntimeLayerState,
        state_index: usize,
        layer_index: usize,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        view_model_triggers: &[StateMachineViewModelTriggerInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
    ) -> Option<(usize, usize)> {
        let mut weighted_candidates = Vec::new();
        let mut total_weight = 0_u64;
        let mut waiting_for_exit = false;

        for (transition_index, transition) in state.transitions.iter().enumerate() {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }

            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index == Some(state_index),
            );
            match transition.allow(
                artboard,
                inputs,
                bindable_numbers,
                bindable_integers,
                bindable_colors,
                bindable_strings,
                bindable_enums,
                bindable_assets,
                bindable_artboards,
                bindable_triggers,
                bindable_view_models,
                bindable_booleans,
                view_model_triggers,
                data_context_present,
                data_context_view_model_bound,
                layer_index,
                animation_from,
            ) {
                TransitionAllowance::No => {}
                TransitionAllowance::WaitingForExit => {
                    waiting_for_exit = true;
                }
                TransitionAllowance::Yes => {
                    total_weight = total_weight.saturating_add(transition.random_weight);
                    weighted_candidates.push((
                        transition_index,
                        state_to_index,
                        transition.random_weight,
                    ));
                }
            }
        }

        if total_weight == 0 {
            self.waiting_for_exit = waiting_for_exit;
            return None;
        }

        let random_weight = Self::random_transition_value() * total_weight as f64;
        let mut current_weight = 0.0_f64;
        for (transition_index, state_to_index, transition_weight) in weighted_candidates {
            current_weight += transition_weight as f64;
            if current_weight > random_weight {
                self.waiting_for_exit = false;
                return Some((transition_index, state_to_index));
            }
        }

        self.waiting_for_exit = waiting_for_exit;
        None
    }

    fn random_transition_value() -> f64 {
        0.0
    }

    fn current_transition_animation<'a>(
        &'a self,
        artboard: &'a ArtboardInstance,
        transition: &RuntimeStateTransition,
        is_current_state: bool,
    ) -> Option<RuntimeTransitionAnimationRef<'a>> {
        if !is_current_state {
            return None;
        }

        if let Some(animation_instance) = self.current_animation.as_ref() {
            let animation = artboard.linear_animation(animation_instance.animation_index)?;
            return Some(RuntimeTransitionAnimationRef {
                instance: animation_instance,
                animation,
            });
        }

        let blend_animation_index = transition.exit_blend_animation_index?;
        let animation_instance = self
            .current_blend_state_1d
            .as_ref()
            .and_then(|blend_state| blend_state.animation_instance(blend_animation_index))
            .or_else(|| {
                self.current_blend_state_direct
                    .as_ref()
                    .and_then(|blend_state| blend_state.animation_instance(blend_animation_index))
            })?;
        let animation = artboard.linear_animation(animation_instance.animation_index)?;
        Some(RuntimeTransitionAnimationRef {
            instance: animation_instance,
            animation,
        })
    }

    fn change_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        transition: &RuntimeStateTransition,
        state_to_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        transition_durations: &[StateMachineTransitionDurationInstance],
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) {
        let previous_state_index = self.current_state_index;
        let mut previous_animation = self.current_animation.clone();
        let previous_blend_state_1d = self.current_blend_state_1d.clone();
        let previous_blend_state_direct = self.current_blend_state_direct.clone();
        let previous_spilled_time = previous_animation
            .as_ref()
            .map(LinearAnimationInstance::spilled_time)
            .unwrap_or(0.0);
        let previous_mix = self.transition_mix;
        if let Some(previous_state) =
            previous_state_index.and_then(|state_index| layer.states.get(state_index))
        {
            previous_state.perform_fire_actions(
                StateMachineFireOccurrence::AtEnd,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
            );
            previous_state.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
                pending_view_model_actions,
            );
        }

        if transition.pause_on_exit()
            && transition.has_exit_time()
            && let Some(animation_instance) = previous_animation.as_mut()
            && let Some(animation) = artboard.linear_animation(animation_instance.animation_index)
        {
            animation_instance.set_time(
                animation,
                transition.exit_time_seconds(Some(animation), true),
            );
        }
        let previous_runtime_animation =
            previous_animation.as_ref().and_then(|animation_instance| {
                artboard.linear_animation(animation_instance.animation_index)
            });
        let duration_override =
            transition_duration_value(transition_durations, transition.global_id);
        let transition_duration_seconds =
            transition.transition_duration_seconds(previous_runtime_animation, duration_override);

        self.current_state_index = Some(state_to_index);
        self.refresh_current_animation(artboard, layer, inputs, bindable_numbers);
        if let Some(current_state) = layer.states.get(state_to_index) {
            current_state.perform_fire_actions(
                StateMachineFireOccurrence::AtStart,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
            );
            current_state.perform_listener_actions(
                StateMachineFireOccurrence::AtStart,
                inputs,
                reported_events,
                pending_view_model_actions,
            );
        }
        transition.perform_fire_actions(
            StateMachineFireOccurrence::AtStart,
            data_context_view_model_bound,
            view_model_triggers,
            reported_events,
        );
        transition.perform_listener_actions(
            StateMachineFireOccurrence::AtStart,
            inputs,
            reported_events,
            pending_view_model_actions,
        );
        if previous_spilled_time != 0.0 {
            self.advance_current_animation(
                artboard,
                layer,
                previous_spilled_time,
                inputs,
                bindable_numbers,
                reported_events,
            );
        }

        if transition_duration_seconds == 0.0 {
            transition.perform_fire_actions(
                StateMachineFireOccurrence::AtEnd,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
            );
            transition.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
                pending_view_model_actions,
            );
            self.clear_transition_source();
            return;
        }

        let mut reset_animation_indices = Vec::new();
        if let Some(animation) = previous_animation.as_ref() {
            reset_animation_indices.push(animation.animation_index);
        }
        if let Some(animation) = self.current_animation.as_ref() {
            reset_animation_indices.push(animation.animation_index);
        }
        let transition_animation_reset =
            AnimationReset::from_animation_indices(artboard, &reset_animation_indices, false);

        if let Some(source_state_index) = previous_state_index {
            self.transition_source_state_index = Some(source_state_index);
            self.transition_source_animation = previous_animation;
            self.transition_source_blend_state_1d = previous_blend_state_1d;
            self.transition_source_blend_state_direct = previous_blend_state_direct;
            self.transition_duration_seconds = transition_duration_seconds;
            self.transition_mix_from = previous_mix;
            self.transition_mix = 0.0;
            self.transition_source_paused = transition.pause_on_exit();
            self.transition_interpolator = transition.interpolator;
            self.transition_enable_early_exit = transition.enable_early_exit();
            self.transition_fire_actions = transition.fire_actions.clone();
            self.transition_listener_actions = transition.listener_actions.clone();
            self.transition_completed = false;
            self.transition_animation_reset = transition_animation_reset;
            self.update_transition_mix(
                0.0,
                inputs,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
                pending_view_model_actions,
            );
        } else {
            self.clear_transition_source();
        }
    }

    fn clear_transition_source(&mut self) {
        self.transition_source_state_index = None;
        self.transition_source_animation = None;
        self.transition_source_blend_state_1d = None;
        self.transition_source_blend_state_direct = None;
        self.transition_duration_seconds = 0.0;
        self.transition_mix = 1.0;
        self.transition_mix_from = 1.0;
        self.transition_source_paused = false;
        self.transition_interpolator = None;
        self.transition_enable_early_exit = false;
        self.transition_fire_actions.clear();
        self.transition_listener_actions.clear();
        self.transition_completed = false;
        self.transition_animation_reset = None;
    }

    fn is_transitioning(&self) -> bool {
        self.has_transition_source()
            && self.transition_duration_seconds != 0.0
            && self.transition_mix < 1.0
    }

    fn has_transition_source(&self) -> bool {
        self.transition_source_state_index.is_some()
    }

    fn update_transition_mix(
        &mut self,
        elapsed_seconds: f32,
        inputs: &mut [StateMachineInputInstance],
        data_context_view_model_bound: bool,
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
        pending_view_model_actions: &mut Vec<RuntimeScheduledListenerAction>,
    ) -> bool {
        if !self.has_transition_source() || self.transition_duration_seconds == 0.0 {
            self.transition_mix = 1.0;
            return false;
        }
        self.transition_mix = (self.transition_mix
            + elapsed_seconds / self.transition_duration_seconds)
            .clamp(0.0, 1.0);
        if self.transition_mix == 1.0 && !self.transition_completed {
            self.transition_completed = true;
            self.transition_animation_reset = None;
            perform_state_machine_fire_actions(
                &self.transition_fire_actions,
                StateMachineFireOccurrence::AtEnd,
                data_context_view_model_bound,
                view_model_triggers,
                reported_events,
            );
            perform_scheduled_listener_actions(
                &self.transition_listener_actions,
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
                pending_view_model_actions,
            )
        } else {
            false
        }
    }

    fn refresh_current_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation = None;
            self.current_blend_state_1d = None;
            self.current_blend_state_direct = None;
            self.current_animation_keep_going = false;
            return;
        };
        if let Some(blend_state) = state.blend_state_1d.as_ref() {
            self.current_animation = None;
            let mut blend_instance =
                BlendState1DInstance::new(blend_state, artboard, state.resets_blend_values());
            blend_instance.advance(artboard, inputs, bindable_numbers, 0.0);
            self.current_blend_state_1d = Some(blend_instance);
            self.current_blend_state_direct = None;
            self.current_animation_keep_going = true;
            return;
        }
        if let Some(blend_state) = state.blend_state_direct.as_ref() {
            self.current_animation = None;
            self.current_blend_state_1d = None;
            let mut blend_instance = BlendStateDirectInstance::new(blend_state, artboard);
            blend_instance.advance(artboard, inputs, bindable_numbers, 0.0);
            self.current_blend_state_direct = Some(blend_instance);
            self.current_animation_keep_going = true;
            return;
        }
        self.current_blend_state_1d = None;
        self.current_blend_state_direct = None;
        let Some(animation_index) = state.animation_index else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let Some(animation) = artboard.linear_animation(animation_index) else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let mut animation_instance =
            LinearAnimationInstance::new(animation_index, animation, state.speed);
        self.current_animation_keep_going = animation_instance.advance(animation, 0.0);
        self.current_animation = Some(animation_instance);
    }

    fn advance_current_animation(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if let Some(blend_state) = self.current_blend_state_1d.as_mut() {
            self.current_animation_keep_going = blend_state.advance_with_events(
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            );
            return self.current_animation_keep_going;
        }
        if let Some(blend_state) = self.current_blend_state_direct.as_mut() {
            self.current_animation_keep_going = blend_state.advance_with_events(
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            );
            return self.current_animation_keep_going;
        }
        let Some(animation_instance) = self.current_animation.as_mut() else {
            self.current_animation_keep_going = false;
            return false;
        };
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation_keep_going = false;
            return false;
        };
        self.current_animation_keep_going = artboard.advance_linear_animation_instance_with_events(
            animation_instance,
            elapsed_seconds * state.speed,
            reported_events,
        );
        self.current_animation_keep_going
    }

    fn advance_transition_source_animation(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if !self.is_transitioning() {
            return false;
        }
        if self.transition_source_paused {
            return false;
        }
        if let Some(blend_state) = self.transition_source_blend_state_1d.as_mut() {
            return blend_state.advance_with_events(
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            );
        }
        if let Some(blend_state) = self.transition_source_blend_state_direct.as_mut() {
            return blend_state.advance_with_events(
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            );
        }
        let Some(animation_instance) = self.transition_source_animation.as_mut() else {
            return false;
        };
        let Some(state) = self
            .transition_source_state_index
            .and_then(|index| layer.states.get(index))
        else {
            return false;
        };
        artboard.advance_linear_animation_instance_with_events(
            animation_instance,
            elapsed_seconds * state.speed,
            reported_events,
        )
    }

    fn apply_animations(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        if self.is_transitioning() {
            if let Some(reset) = self.transition_animation_reset.as_ref() {
                changed |= reset.apply(artboard);
            }
            let mix_from = self
                .transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix_from))
                .unwrap_or(self.transition_mix_from);
            if let Some(source_animation) = self.transition_source_animation.as_ref() {
                changed |= artboard.apply_linear_animation_instance(source_animation, mix_from);
            }
            if let Some(source_blend_state) = self.transition_source_blend_state_1d.as_ref() {
                changed |= source_blend_state.apply(artboard, mix_from);
            }
            if let Some(source_blend_state) = self.transition_source_blend_state_direct.as_ref() {
                changed |= source_blend_state.apply(artboard, mix_from);
            }
        }
        let Some(animation_instance) = self.current_animation.as_ref() else {
            if let Some(blend_state) = self.current_blend_state_1d.as_ref() {
                let mix = if self.has_transition_source() {
                    self.transition_interpolator
                        .map(|interpolator| interpolator.transform(self.transition_mix))
                        .unwrap_or(self.transition_mix)
                } else {
                    1.0
                };
                return changed | blend_state.apply(artboard, mix);
            }
            if let Some(blend_state) = self.current_blend_state_direct.as_ref() {
                let mix = if self.has_transition_source() {
                    self.transition_interpolator
                        .map(|interpolator| interpolator.transform(self.transition_mix))
                        .unwrap_or(self.transition_mix)
                } else {
                    1.0
                };
                return changed | blend_state.apply(artboard, mix);
            }
            return changed;
        };
        let mix = if self.has_transition_source() {
            self.transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix))
                .unwrap_or(self.transition_mix)
        } else {
            1.0
        };
        changed | artboard.apply_linear_animation_instance(animation_instance, mix)
    }
}
