use super::{RuntimeTransitionInputCondition, StateMachineInputInstance};
use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionTriggerCondition {
    input: RuntimeTransitionInputCondition,
}

impl RuntimeTransitionTriggerCondition {
    pub(super) fn from_object(
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        Some(Self {
            input: RuntimeTransitionInputCondition::from_object(
                state_machine_inputs,
                "StateMachineTrigger",
                object,
            )?,
        })
    }

    pub(super) fn evaluate(self, inputs: &[StateMachineInputInstance], layer_index: usize) -> bool {
        let Some(input) = inputs.get(self.input.input_index()) else {
            return true;
        };
        input
            .trigger_is_fireable_for_layer(layer_index)
            .unwrap_or(true)
    }

    pub(super) fn use_input(self, inputs: &mut [StateMachineInputInstance], layer_index: usize) {
        if let Some(input) = inputs.get_mut(self.input.input_index()) {
            input.use_trigger_in_layer(layer_index);
        }
    }
}
