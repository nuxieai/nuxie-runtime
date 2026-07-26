use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    pub(super) value: StateMachineInputValue,
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
pub(super) enum StateMachineInputValue {
    Bool(bool),
    Number(f32),
    Trigger {
        fired: bool,
        used_layers: Vec<usize>,
    },
}
