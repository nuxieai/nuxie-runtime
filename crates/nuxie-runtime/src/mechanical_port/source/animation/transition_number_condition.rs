use crate::mechanical_port::source::{
    animation::{
        state_machine_input_instance::SMINumber, transition_condition_op::TransitionConditionOp,
    },
    generated::animation::transition_number_condition_base::TransitionNumberConditionBase,
};

pub trait NumberConditionStateMachine {
    fn number_input(&self, id: u32) -> Option<&SMINumber>;
}
pub trait NumberInputKind {
    fn is_state_machine_number(&self) -> bool;
}

#[derive(Default)]
pub struct TransitionNumberCondition {
    pub base: TransitionNumberConditionBase,
}

impl TransitionNumberCondition {
    pub fn validate_input_type(&self, input: Option<&dyn NumberInputKind>) -> bool {
        input.is_none() || input.is_some_and(NumberInputKind::is_state_machine_number)
    }
    pub fn evaluate(&self, machine: &dyn NumberConditionStateMachine, _layer: *mut ()) -> bool {
        let Some(input) = machine.number_input(self.base.base.base.input_id()) else {
            return true;
        };
        match self.base.base.op() {
            TransitionConditionOp::Equal => input.value() == self.base.value(),
            TransitionConditionOp::NotEqual => input.value() != self.base.value(),
            TransitionConditionOp::LessThanOrEqual => input.value() <= self.base.value(),
            TransitionConditionOp::LessThan => input.value() < self.base.value(),
            TransitionConditionOp::GreaterThanOrEqual => input.value() >= self.base.value(),
            TransitionConditionOp::GreaterThan => input.value() > self.base.value(),
        }
    }
}
