use crate::mechanical_port::source::{
    animation::{state_machine::StateMachine, state_machine_input::StateMachineInput},
    generated::animation::transition_input_condition_base::TransitionInputConditionBase,
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    status_code::StatusCode,
};

pub trait TransitionInputStateMachine {
    fn input_count(&self) -> usize;
    fn input(&self, index: usize) -> Option<&StateMachineInput>;
}

#[derive(Default)]
pub struct TransitionInputCondition {
    pub base: TransitionInputConditionBase,
}

impl TransitionInputCondition {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<StateMachineImporter>(StateMachine::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let state_machine = importer.state_machine();
        let input_id = self.base.input_id() as usize;
        unsafe {
            if input_id >= state_machine.as_ref().input_count() {
                return StatusCode::InvalidObject;
            }
            if !self.validate_input_type(state_machine.as_ref().input(input_id)) {
                return StatusCode::InvalidObject;
            }
        }
        self.base.base.import(import_stack)
    }

    pub fn validate_input_type(&self, _input: Option<&StateMachineInput>) -> bool {
        true
    }
}
