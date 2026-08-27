use crate::mechanical_port::source::{
    animation::{nested_input::NestedInput, state_machine_input::StateMachineInput},
    generated::animation::listener_input_change_base::ListenerInputChangeBase,
    importers::{
        artboard_importer::ArtboardImporter, import_stack::ImportStack,
        state_machine_importer::StateMachineImporter,
    },
    status_code::StatusCode,
};
pub trait ListenerInputTypeValidation {
    fn validate_input_type(&self, input: Option<&StateMachineInput>) -> bool {
        let _ = input;
        true
    }
    fn validate_nested_input_type(&self, input: Option<&NestedInput>) -> bool {
        let _ = input;
        true
    }
}
#[derive(Default)]
pub struct ListenerInputChange {
    pub base: ListenerInputChangeBase,
}
impl ListenerInputTypeValidation for ListenerInputChange {}
impl ListenerInputChange {
    pub fn import_with_validation(
        &mut self,
        stack: &mut ImportStack,
        validation: &dyn ListenerInputTypeValidation,
    ) -> StatusCode {
        let Some(machine_importer) = stack.latest::<StateMachineImporter>(crate::mechanical_port::source::generated::animation::state_machine_base::StateMachineBase::TYPE_KEY) else { return StatusCode::MissingObject };
        let machine = machine_importer.state_machine();
        let Some(artboard_importer) = stack.latest::<ArtboardImporter>(
            crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let nested = unsafe {
            artboard_importer
                .artboard()
                .as_ref()
                .resolve_nested_input(self.base.nested_input_id())
        };
        if let Some(nested) = nested {
            if !validation.validate_nested_input_type(Some(nested)) {
                return StatusCode::InvalidObject;
            }
        } else {
            let input = unsafe { machine.as_ref().input(self.base.input_id() as usize) };
            if !validation.validate_input_type(input) {
                return StatusCode::InvalidObject;
            }
        }
        self.base.base.import(stack)
    }
}
