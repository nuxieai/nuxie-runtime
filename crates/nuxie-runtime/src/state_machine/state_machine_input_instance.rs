use super::state_machine_input::{
    RuntimeStateMachineInput, RuntimeStateMachineInputHandle, StateMachineInputDefaultValue,
    StateMachineInputKind,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StateMachineInputInstance {
    index: usize,
    definition: RuntimeStateMachineInputHandle,
    value: StateMachineInputInstanceValue,
}

impl StateMachineInputInstance {
    pub(crate) fn new(index: usize, inputs: Arc<Vec<RuntimeStateMachineInput>>) -> Self {
        let definition = RuntimeStateMachineInputHandle::new(inputs, index);
        let value = match definition.definition().value {
            StateMachineInputDefaultValue::Bool(value) => {
                StateMachineInputInstanceValue::Bool(value)
            }
            StateMachineInputDefaultValue::Number(value) => {
                StateMachineInputInstanceValue::Number(value)
            }
            StateMachineInputDefaultValue::Trigger => StateMachineInputInstanceValue::Trigger {
                fired: false,
                used_layers: Vec::new(),
            },
        };
        Self {
            index,
            definition,
            value,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn global_id(&self) -> u32 {
        self.definition.definition().global_id
    }

    pub fn name(&self) -> Option<&str> {
        self.definition.definition().name.as_deref()
    }

    pub fn kind(&self) -> StateMachineInputKind {
        self.definition.definition().kind
    }

    pub fn bool_value(&self) -> Option<bool> {
        match self.value {
            StateMachineInputInstanceValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn number_value(&self) -> Option<f32> {
        match self.value {
            StateMachineInputInstanceValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn trigger_fired(&self) -> Option<bool> {
        match self.value {
            StateMachineInputInstanceValue::Trigger { fired, .. } => Some(fired),
            _ => None,
        }
    }

    pub(crate) fn set_bool(&mut self, value: bool) -> bool {
        match &mut self.value {
            StateMachineInputInstanceValue::Bool(current) => {
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
            StateMachineInputInstanceValue::Number(current) => {
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
            StateMachineInputInstanceValue::Bool(current) => {
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
            StateMachineInputInstanceValue::Trigger { fired, .. } => {
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
            StateMachineInputInstanceValue::Trigger { fired, used_layers } => {
                Some(*fired && !used_layers.contains(&layer_index))
            }
            _ => None,
        }
    }

    pub(crate) fn use_trigger_in_layer(&mut self, layer_index: usize) {
        if let StateMachineInputInstanceValue::Trigger { used_layers, .. } = &mut self.value
            && !used_layers.contains(&layer_index)
        {
            used_layers.push(layer_index);
        }
    }

    pub(crate) fn advanced(&mut self) {
        if let StateMachineInputInstanceValue::Trigger { fired, used_layers } = &mut self.value {
            *fired = false;
            used_layers.clear();
        }
    }
}

#[derive(Debug, Clone)]
enum StateMachineInputInstanceValue {
    Bool(bool),
    Number(f32),
    Trigger {
        fired: bool,
        used_layers: Vec<usize>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_occurrence_retains_the_authored_definition_arena() {
        let definitions = Arc::new(vec![RuntimeStateMachineInput::new_number(
            42,
            Some("speed".to_owned()),
            3.5,
        )]);
        let instance = StateMachineInputInstance::new(0, Arc::clone(&definitions));

        assert_eq!(Arc::strong_count(&definitions), 2);
        assert_eq!(instance.global_id(), 42);
        assert_eq!(instance.name(), Some("speed"));
        assert_eq!(instance.kind(), StateMachineInputKind::Number);
        assert_eq!(instance.number_value(), Some(3.5));
    }
}
