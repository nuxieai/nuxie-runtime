use super::{RuntimeTransitionInputCondition, StateMachineInputInstance};
use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransitionConditionOp {
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
}

impl TransitionConditionOp {
    pub(super) fn from_value(value: u64) -> Self {
        match value {
            1 => Self::NotEqual,
            2 => Self::LessThanOrEqual,
            3 => Self::GreaterThanOrEqual,
            4 => Self::LessThan,
            5 => Self::GreaterThan,
            _ => Self::Equal,
        }
    }

    pub(super) fn compare(self, input_value: f32, value: f32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            Self::LessThanOrEqual => input_value <= value,
            Self::GreaterThanOrEqual => input_value >= value,
            Self::LessThan => input_value < value,
            Self::GreaterThan => input_value > value,
        }
    }

    pub(super) fn compare_bool(self, input_value: bool, value: bool) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    pub(super) fn compare_u32_equal_only(self, input_value: u32, value: u32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    pub(super) fn compare_bytes_equal_only(self, input_value: &[u8], value: &[u8]) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionNumberCondition {
    input: RuntimeTransitionInputCondition,
    // Pinned `TransitionValueCondition::op()` casts the retained uint property
    // without normalizing unknown values. Keep that raw value so the switch in
    // `TransitionNumberCondition::evaluate` reaches its final `return false`.
    op_value: u32,
    value: f32,
}

impl RuntimeTransitionNumberCondition {
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
                "StateMachineNumber",
                object,
            )?,
            op_value,
            value: object.double_property("value").unwrap_or(0.0),
        })
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize, op: TransitionConditionOp, value: f32) -> Self {
        Self {
            input: RuntimeTransitionInputCondition::new(input_index),
            op_value: match op {
                TransitionConditionOp::Equal => 0,
                TransitionConditionOp::NotEqual => 1,
                TransitionConditionOp::LessThanOrEqual => 2,
                TransitionConditionOp::GreaterThanOrEqual => 3,
                TransitionConditionOp::LessThan => 4,
                TransitionConditionOp::GreaterThan => 5,
            },
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
        match self.op_value {
            0 => input_value == self.value,
            1 => input_value != self.value,
            2 => input_value <= self.value,
            3 => input_value >= self.value,
            4 => input_value < self.value,
            5 => input_value > self.value,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::state_machine_input::RuntimeStateMachineInput;
    use super::*;
    use std::sync::Arc;

    fn number_input(value: f32) -> StateMachineInputInstance {
        StateMachineInputInstance::new(
            0,
            Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
                1, None, value,
            ))]),
        )
    }

    #[test]
    fn unknown_operation_does_not_normalize_to_equal() {
        let condition = RuntimeTransitionNumberCondition {
            input: RuntimeTransitionInputCondition::new(0),
            op_value: 6,
            value: 7.0,
        };

        assert!(!condition.evaluate(&[number_input(7.0)]));
        assert!(!condition.evaluate(&[number_input(8.0)]));
    }
}
