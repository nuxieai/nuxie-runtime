use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    core::Core,
    generated::animation::listener_number_change_base::ListenerNumberChangeBase,
};
pub trait NumberChangeInputKind {
    fn is_number(&self) -> bool;
    fn is_nested_number(&self) -> bool;
}
#[derive(Default)]
pub struct ListenerNumberChange {
    pub base: ListenerNumberChangeBase,
}
impl ListenerNumberChange {
    pub fn validate_input_type(&self, input: Option<&dyn NumberChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(NumberChangeInputKind::is_number)
    }
    pub fn validate_nested_input_type(&self, input: Option<&dyn NumberChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(NumberChangeInputKind::is_nested_number)
    }
    pub fn perform(&self, machine: &mut StateMachineInstance, _invocation: &ListenerInvocation) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            machine.set_nested_number(self.base.base.nested_input_id(), self.base.value());
        } else {
            if let Some(input) = machine.number_input_mut(self.base.base.input_id()) {
                input.set_value(self.base.value());
            }
        }
    }
}

impl std::ops::Deref for ListenerNumberChange {
    type Target = ListenerNumberChangeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerNumberChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
