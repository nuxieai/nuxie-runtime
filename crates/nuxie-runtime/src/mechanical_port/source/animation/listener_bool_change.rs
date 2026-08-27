use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, core::Core,
    generated::animation::listener_bool_change_base::ListenerBoolChangeBase,
};
pub trait BoolChangeStateMachine {
    fn nested_bool(&self, id: u32) -> Option<bool>;
    fn set_nested_bool(&mut self, id: u32, value: bool);
    fn bool_input(&self, id: u32) -> Option<bool>;
    fn set_bool_input(&mut self, id: u32, value: bool);
}
pub trait BoolChangeInputKind {
    fn is_bool(&self) -> bool;
    fn is_nested_bool(&self) -> bool;
}
#[derive(Default)]
pub struct ListenerBoolChange {
    pub base: ListenerBoolChangeBase,
}
impl ListenerBoolChange {
    pub fn validate_input_type(&self, input: Option<&dyn BoolChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(BoolChangeInputKind::is_bool)
    }
    pub fn validate_nested_input_type(&self, input: Option<&dyn BoolChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(BoolChangeInputKind::is_nested_bool)
    }
    fn changed_value(&self, current: bool) -> bool {
        match self.base.value() {
            0 => false,
            1 => true,
            _ => !current,
        }
    }
    pub fn perform(
        &self,
        machine: &mut dyn BoolChangeStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            let id = self.base.base.nested_input_id();
            if let Some(current) = machine.nested_bool(id) {
                machine.set_nested_bool(id, self.changed_value(current));
            }
        } else {
            let id = self.base.base.input_id();
            if let Some(current) = machine.bool_input(id) {
                machine.set_bool_input(id, self.changed_value(current));
            }
        }
    }
}
