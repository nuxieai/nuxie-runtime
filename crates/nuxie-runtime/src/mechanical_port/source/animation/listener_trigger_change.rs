use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, core::Core,
    generated::animation::listener_trigger_change_base::ListenerTriggerChangeBase,
};
pub trait TriggerChangeStateMachine {
    fn fire_nested_trigger(&mut self, id: u32, delay: f32) -> bool;
    fn fire_trigger_input(&mut self, id: u32) -> bool;
}
pub trait TriggerChangeInputKind {
    fn is_trigger(&self) -> bool;
    fn is_nested_trigger(&self) -> bool;
}
#[derive(Default)]
pub struct ListenerTriggerChange {
    pub base: ListenerTriggerChangeBase,
}
impl ListenerTriggerChange {
    pub fn validate_input_type(&self, input: Option<&dyn TriggerChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(TriggerChangeInputKind::is_trigger)
    }
    pub fn validate_nested_input_type(&self, input: Option<&dyn TriggerChangeInputKind>) -> bool {
        input.is_none() || input.is_some_and(TriggerChangeInputKind::is_nested_trigger)
    }
    pub fn perform(
        &self,
        machine: &mut dyn TriggerChangeStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            machine.fire_nested_trigger(self.base.base.nested_input_id(), 0.0);
        } else {
            machine.fire_trigger_input(self.base.base.input_id());
        }
    }
}
