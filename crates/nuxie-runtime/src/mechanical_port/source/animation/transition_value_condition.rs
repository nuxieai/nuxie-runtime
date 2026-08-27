use crate::mechanical_port::source::{
    animation::transition_condition_op::TransitionConditionOp,
    generated::animation::transition_value_condition_base::TransitionValueConditionBase,
};

#[derive(Default)]
pub struct TransitionValueCondition {
    pub base: TransitionValueConditionBase,
}

impl TransitionValueCondition {
    pub fn op(&self) -> TransitionConditionOp {
        match self.base.op_value() {
            0 => TransitionConditionOp::Equal,
            1 => TransitionConditionOp::NotEqual,
            2 => TransitionConditionOp::LessThanOrEqual,
            3 => TransitionConditionOp::GreaterThanOrEqual,
            4 => TransitionConditionOp::LessThan,
            5 => TransitionConditionOp::GreaterThan,
            value => unreachable!("invalid transition condition operation {value}"),
        }
    }
}
