use crate::mechanical_port::source::{
    animation::{
        state_machine_input_instance::SMIBool,
        state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
        transition_condition_op::TransitionConditionOp,
    },
    generated::animation::state_machine_bool_base::StateMachineBoolBase,
    generated::animation::transition_bool_condition_base::TransitionBoolConditionBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
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
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.import_with(import_stack, |input| {
            input.is_type_of(StateMachineBoolBase::TYPE_KEY)
        })
    }

    pub fn validate_input_type(&self, input: Option<&dyn StateMachineInputKind>) -> bool {
        // A null input is valid so old runtimes can limp along when a newer
        // input type is introduced; evaluation then returns true.
        input.is_none() || input.is_some_and(StateMachineInputKind::is_state_machine_bool)
    }

    pub fn evaluate(
        &self,
        state_machine_instance: &dyn BoolConditionStateMachine,
        _layer_instance: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(bool_input) = state_machine_instance.bool_input(self.base.base.base.input_id())
        else {
            return true;
        };
        (bool_input.value() && self.base.base.op() == TransitionConditionOp::Equal)
            || (!bool_input.value() && self.base.base.op() == TransitionConditionOp::NotEqual)
    }
}
