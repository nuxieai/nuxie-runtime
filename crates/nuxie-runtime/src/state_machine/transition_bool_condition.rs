#[cfg(test)]
use super::TransitionConditionOp;
use super::{RuntimeTransitionInputCondition, StateMachineInputInstance};
use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionBoolCondition {
    input: RuntimeTransitionInputCondition,
    // Pinned `TransitionValueCondition::op()` casts the retained uint property
    // without normalizing unknown values. Keep that raw value so an unknown
    // operation falls through both comparisons and evaluates false.
    op_value: u32,
}

impl RuntimeTransitionBoolCondition {
    pub(super) fn from_object(
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        let op_value = match object.uint_property("opValue") {
            Some(value) => u32::try_from(value).ok()?,
            None => 0,
        };
        Some(Self {
            input: RuntimeTransitionInputCondition::from_object(
                state_machine_inputs,
                "StateMachineBool",
                object,
            )?,
            op_value,
        })
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize, op: TransitionConditionOp) -> Self {
        Self {
            input: RuntimeTransitionInputCondition::new(input_index),
            op_value: match op {
                TransitionConditionOp::Equal => 0,
                TransitionConditionOp::NotEqual => 1,
                TransitionConditionOp::LessThanOrEqual => 2,
                TransitionConditionOp::GreaterThanOrEqual => 3,
                TransitionConditionOp::LessThan => 4,
                TransitionConditionOp::GreaterThan => 5,
                TransitionConditionOp::Unsupported => u32::MAX,
            },
        }
    }

    pub(super) fn evaluate(self, inputs: &[StateMachineInputInstance]) -> bool {
        let Some(value) = inputs
            .get(self.input.input_index())
            .and_then(StateMachineInputInstance::bool_value)
        else {
            return true;
        };
        (value && self.op_value == 0) || (!value && self.op_value == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::super::state_machine_input::RuntimeStateMachineInput;
    use super::*;
    use std::sync::Arc;

    fn bool_input(value: bool) -> StateMachineInputInstance {
        StateMachineInputInstance::new(
            0,
            Arc::new(vec![Some(RuntimeStateMachineInput::new_bool(
                1, None, value,
            ))]),
        )
    }

    #[test]
    fn unknown_operation_does_not_normalize_to_equal() {
        let condition = RuntimeTransitionBoolCondition {
            input: RuntimeTransitionInputCondition::new(0),
            op_value: 6,
        };

        assert!(!condition.evaluate(&[bool_input(true)]));
        assert!(!condition.evaluate(&[bool_input(false)]));
    }
}
