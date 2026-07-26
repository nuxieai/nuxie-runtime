use nuxie_binary::RuntimeObject;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    pub(super) value: StateMachineInputDefaultValue,
}

impl RuntimeStateMachineInput {
    pub(crate) fn new_bool(global_id: u32, name: Option<String>, value: bool) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Bool,
            value: StateMachineInputDefaultValue::Bool(value),
        }
    }

    pub(crate) fn new_number(global_id: u32, name: Option<String>, value: f32) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Number,
            value: StateMachineInputDefaultValue::Number(value),
        }
    }

    pub(crate) fn new_trigger(global_id: u32, name: Option<String>) -> Self {
        Self {
            global_id,
            name,
            kind: StateMachineInputKind::Trigger,
            value: StateMachineInputDefaultValue::Trigger,
        }
    }
}

pub(super) fn runtime_state_machine_input(
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
pub(super) enum StateMachineInputDefaultValue {
    Bool(bool),
    Number(f32),
    Trigger,
}

/// Stable reference to one authored input in the StateMachine-owned arena.
///
/// Pinned C++ stores `const StateMachineInput* m_input` on every `SMIInput`
/// (`state_machine_input_instance.hpp:35,40-42`) and reads the name/type
/// through that retained definition (`state_machine_input_instance.cpp:14-16`).
#[derive(Debug, Clone)]
pub(super) struct RuntimeStateMachineInputHandle {
    inputs: Arc<Vec<RuntimeStateMachineInput>>,
    index: usize,
}

impl RuntimeStateMachineInputHandle {
    pub(super) fn new(inputs: Arc<Vec<RuntimeStateMachineInput>>, index: usize) -> Self {
        debug_assert!(index < inputs.len());
        Self { inputs, index }
    }

    pub(super) fn definition(&self) -> &RuntimeStateMachineInput {
        &self.inputs[self.index]
    }
}
