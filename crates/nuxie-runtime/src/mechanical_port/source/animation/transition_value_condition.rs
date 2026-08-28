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
impl std::ops::Deref for TransitionValueCondition {
    type Target = TransitionValueConditionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionValueCondition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::transition_input_condition_base::TransitionInputConditionBaseCallbacks for TransitionValueCondition { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
impl crate::mechanical_port::source::generated::animation::transition_value_condition_base::TransitionValueConditionBaseCallbacks for TransitionValueCondition { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
