use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    core::Core,
    generated::animation::listener_trigger_change_base::ListenerTriggerChangeBase,
};
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
    pub fn perform(&self, machine: &mut StateMachineInstance, _invocation: &ListenerInvocation) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            machine.fire_nested_trigger(self.base.base.nested_input_id());
        } else {
            if let Some(input) = machine.trigger_input_mut(self.base.base.input_id()) {
                input.fire();
            }
        }
    }
}
