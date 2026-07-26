use super::{RuntimeTransitionInputCondition, StateMachineInputInstance, TransitionConditionOp};
use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionNumberCondition {
    input: RuntimeTransitionInputCondition,
    op: TransitionConditionOp,
    value: f32,
}

impl RuntimeTransitionNumberCondition {
    pub(super) fn from_object(
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        Some(Self {
            input: RuntimeTransitionInputCondition::from_object(
                state_machine_inputs,
                "StateMachineNumber",
                object,
            )?,
            op: TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
            value: object.double_property("value").unwrap_or(0.0),
        })
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize, op: TransitionConditionOp, value: f32) -> Self {
        Self {
            input: RuntimeTransitionInputCondition::new(input_index),
            op,
            value,
        }
    }

    pub(super) fn evaluate(self, inputs: &[StateMachineInputInstance]) -> bool {
        let Some(input_value) = inputs
            .get(self.input.input_index())
            .and_then(StateMachineInputInstance::number_value)
        else {
            return true;
        };
        self.op.compare(input_value, self.value)
    }
}
