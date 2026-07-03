use crate::animation::{LinearAnimationInstance, RuntimeInterpolator};
use crate::{ArtboardInstance, StateMachineBindableNumberInstance, bindable_number_value};
use rive_binary::{RuntimeFile, RuntimeObject};
use std::collections::BTreeMap;

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
}

impl BlendState1DInstance {
    pub(crate) fn new(blend_state: &RuntimeBlendState1D, artboard: &ArtboardInstance) -> Self {
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

        Self {
            source: blend_state.source.clone(),
            animations,
        }
    }

    pub(crate) fn advance(
        &mut self,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_and_report(artboard, inputs, bindable_numbers, elapsed_seconds, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
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
        self.advance_and_report(artboard, inputs, bindable_numbers, elapsed_seconds, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
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
}

impl RuntimeScheduledListenerAction {
    pub(crate) fn from_imported(action: &rive_binary::RuntimeListenerAction<'_>) -> Option<Self> {
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
            _ => None,
        }
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
) -> bool {
    let mut changed_input = false;
    for action in listener_actions {
        let flags = match action {
            RuntimeScheduledListenerAction::FireEvent { flags, .. }
            | RuntimeScheduledListenerAction::BoolChange { flags, .. }
            | RuntimeScheduledListenerAction::NumberChange { flags, .. }
            | RuntimeScheduledListenerAction::TriggerChange { flags, .. } => *flags,
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
