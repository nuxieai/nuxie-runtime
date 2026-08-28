use crate::mechanical_port::source::{
    animation::{
        state_machine_input_instance::SMINumber,
        state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
        transition_condition_op::TransitionConditionOp,
    },
    generated::animation::state_machine_number_base::StateMachineNumberBase,
    generated::animation::transition_number_condition_base::TransitionNumberConditionBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
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
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.import_with(import_stack, |input| {
            input.is_type_of(StateMachineNumberBase::TYPE_KEY)
        })
    }

    pub fn validate_input_type(&self, input: Option<&dyn NumberInputKind>) -> bool {
        input.is_none() || input.is_some_and(NumberInputKind::is_state_machine_number)
    }
    pub fn evaluate(
        &self,
        machine: &dyn NumberConditionStateMachine,
        _layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
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
