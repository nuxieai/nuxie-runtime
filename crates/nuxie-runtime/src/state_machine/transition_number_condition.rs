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
