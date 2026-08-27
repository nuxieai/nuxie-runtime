use crate::mechanical_port::source::{
    animation::{
        state_machine_input_instance::SMIBool, transition_condition_op::TransitionConditionOp,
    },
    generated::animation::transition_bool_condition_base::TransitionBoolConditionBase,
};

pub trait StateMachineInputKind {
    fn is_state_machine_bool(&self) -> bool;
}

pub trait BoolConditionStateMachine {
    fn bool_input(&self, input_id: u32) -> Option<&SMIBool>;
}

#[derive(Default)]
pub struct TransitionBoolCondition {
    pub base: TransitionBoolConditionBase,
}

impl TransitionBoolCondition {
    pub fn validate_input_type(&self, input: Option<&dyn StateMachineInputKind>) -> bool {
        // A null input is valid so old runtimes can limp along when a newer
        // input type is introduced; evaluation then returns true.
        input.is_none() || input.is_some_and(StateMachineInputKind::is_state_machine_bool)
    }

    pub fn evaluate(
        &self,
        state_machine_instance: &dyn BoolConditionStateMachine,
        _layer_instance: *mut (),
    ) -> bool {
        let Some(bool_input) = state_machine_instance.bool_input(self.base.base.base.input_id())
        else {
            return true;
        };
        (bool_input.value() && self.base.base.op() == TransitionConditionOp::Equal)
            || (!bool_input.value() && self.base.base.op() == TransitionConditionOp::NotEqual)
    }
}
