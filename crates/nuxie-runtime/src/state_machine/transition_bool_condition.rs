use super::{RuntimeTransitionInputCondition, StateMachineInputInstance, TransitionConditionOp};
use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionBoolCondition {
    input: RuntimeTransitionInputCondition,
    op: TransitionConditionOp,
}

impl RuntimeTransitionBoolCondition {
    pub(super) fn from_object(
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        Some(Self {
            input: RuntimeTransitionInputCondition::from_object(
                state_machine_inputs,
                "StateMachineBool",
                object,
            )?,
            op: TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
        })
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize, op: TransitionConditionOp) -> Self {
        Self {
            input: RuntimeTransitionInputCondition::new(input_index),
            op,
        }
    }

    pub(super) fn evaluate(self, inputs: &[StateMachineInputInstance]) -> bool {
        let Some(value) = inputs
            .get(self.input.input_index())
            .and_then(StateMachineInputInstance::bool_value)
        else {
            return true;
        };
        (value && self.op == TransitionConditionOp::Equal)
            || (!value && self.op == TransitionConditionOp::NotEqual)
    }
}
