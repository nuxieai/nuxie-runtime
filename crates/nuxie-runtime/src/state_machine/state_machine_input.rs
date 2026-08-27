use nuxie_binary::RuntimeObject;
use std::sync::Arc;

// Mirrors src/animation/state_machine_input.cpp and
// include/rive/animation/state_machine_input.hpp.
#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    pub(super) value: StateMachineInputDefaultValue,
}

impl RuntimeStateMachineInput {
    /// Value exposed by the generated `StateMachineBoolBase` getter.
    pub(crate) fn bool_value(&self) -> Option<bool> {
        match self.value {
            StateMachineInputDefaultValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    /// Value exposed by the generated `StateMachineNumberBase` getter.
    pub(crate) fn number_value(&self) -> Option<f32> {
        match self.value {
            StateMachineInputDefaultValue::Number(value) => Some(value),
            _ => None,
        }
    }

    /// Mechanical translation of `StateMachineInput::onAddedDirty`.
    #[allow(dead_code)]
    pub(crate) fn on_added_dirty(&self) -> bool {
        true
    }

    /// Mechanical translation of `StateMachineInput::onAddedClean`.
    #[allow(dead_code)]
    pub(crate) fn on_added_clean(&self) -> bool {
        true
    }

    pub(crate) fn new_bool(global_id: u32, name: Option<String>, value: bool) -> Self {
        Self {
            global_id,
            // `CoreString` defaults to the empty string in pinned C++; keep a
            // present retained name even when the property was omitted.
            name: Some(name.unwrap_or_default()),
            kind: StateMachineInputKind::Bool,
            value: StateMachineInputDefaultValue::Bool(value),
        }
    }

    pub(crate) fn new_number(global_id: u32, name: Option<String>, value: f32) -> Self {
        Self {
            global_id,
            name: Some(name.unwrap_or_default()),
            kind: StateMachineInputKind::Number,
            value: StateMachineInputDefaultValue::Number(value),
        }
    }

    pub(crate) fn new_trigger(global_id: u32, name: Option<String>) -> Self {
        Self {
            global_id,
            name: Some(name.unwrap_or_default()),
            kind: StateMachineInputKind::Trigger,
            value: StateMachineInputDefaultValue::Trigger,
        }
    }
}

pub(super) fn runtime_state_machine_input(
    object: &RuntimeObject,
) -> Option<RuntimeStateMachineInput> {
    // `StateMachineInput::import` first requires the latest
    // `StateMachineImporter`, then transfers the input into that machine.
    // The binary import-stack pass performs that exact validation and ordered
    // transfer before this immutable runtime definition is constructed.
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
    inputs: Arc<Vec<Option<RuntimeStateMachineInput>>>,
    index: usize,
}

impl RuntimeStateMachineInputHandle {
    pub(super) fn new(inputs: Arc<Vec<Option<RuntimeStateMachineInput>>>, index: usize) -> Self {
        debug_assert!(index < inputs.len());
        Self { inputs, index }
    }

    pub(super) fn definition(&self) -> Option<&RuntimeStateMachineInput> {
        self.inputs[self.index].as_ref()
    }
}
