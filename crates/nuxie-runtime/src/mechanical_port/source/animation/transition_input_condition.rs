use crate::mechanical_port::source::{
    animation::state_machine::StateMachine,
    core::CoreHandle,
    generated::animation::transition_input_condition_base::TransitionInputConditionBase,
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct TransitionInputCondition {
    pub base: TransitionInputConditionBase,
}

impl TransitionInputCondition {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.import_with(import_stack, |_| true)
    }

    pub fn import_with(
        &mut self,
        import_stack: &mut ImportStack,
        validate_input: impl FnOnce(&CoreHandle) -> bool,
    ) -> StatusCode {
        let Some(importer) = import_stack.latest::<StateMachineImporter>(crate::mechanical_port::source::generated::animation::state_machine_base::StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let state_machine = importer.state_machine();
        let input_id = self.base.input_id() as usize;
        let Some(input) = state_machine
            .with_downcast::<StateMachine, _>(|state_machine| state_machine.input(input_id))
            .flatten()
        else {
            return StatusCode::InvalidObject;
        };
        if !validate_input(&input) {
            return StatusCode::InvalidObject;
        }
        self.base.base.import(import_stack)
    }
}
impl std::ops::Deref for TransitionInputCondition {
    type Target = TransitionInputConditionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionInputCondition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::transition_input_condition_base::TransitionInputConditionBaseCallbacks for TransitionInputCondition { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
