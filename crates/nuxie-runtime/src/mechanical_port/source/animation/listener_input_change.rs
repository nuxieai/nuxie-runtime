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
        let nested = artboard_importer
            .artboard()
            .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(|artboard| {
                artboard.resolve_handle(self.base.nested_input_id())
            })
            .flatten();
        if let Some(nested) =
            nested.filter(|nested| nested.is_type_of(crate::mechanical_port::source::generated::animation::nested_input_base::NestedInputBase::TYPE_KEY))
        {
            let valid = nested
                .with_downcast::<NestedInput, _>(|nested| {
                    validation.validate_nested_input_type(Some(nested))
                })
                .unwrap_or(false);
            if !valid {
                return StatusCode::InvalidObject;
            }
        } else {
            let input = machine
                .with_downcast::<crate::mechanical_port::source::animation::state_machine::StateMachine, _>(|machine| {
                    machine.input(self.base.input_id() as usize)
                })
                .flatten();
            let valid = input
                .as_ref()
                .and_then(|input| {
                    input.with_downcast::<StateMachineInput, _>(|input| {
                        validation.validate_input_type(Some(input))
                    })
                })
                .unwrap_or_else(|| validation.validate_input_type(None));
            if !valid {
                return StatusCode::InvalidObject;
            }
        }
        self.base.base.import(stack)
    }
}
impl std::ops::Deref for ListenerInputChange {
    type Target = ListenerInputChangeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerInputChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks for ListenerInputChange { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
impl crate::mechanical_port::source::generated::animation::listener_input_change_base::ListenerInputChangeBaseCallbacks for ListenerInputChange { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
