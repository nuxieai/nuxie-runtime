use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, core::Core,
    generated::animation::listener_number_change_base::ListenerNumberChangeBase,
};
pub trait NumberChangeStateMachine {
    fn set_nested_number(&mut self, id: u32, value: f32) -> bool;
    fn set_number_input(&mut self, id: u32, value: f32) -> bool;
}
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
    pub fn perform(
        &self,
        machine: &mut dyn NumberChangeStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            machine.set_nested_number(self.base.base.nested_input_id(), self.base.value());
        } else {
            machine.set_number_input(self.base.base.input_id(), self.base.value());
        }
    }
}
