use rive_binary::{RuntimeFile, RuntimeObject};

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
