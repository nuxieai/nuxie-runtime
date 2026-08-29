use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    core::Core,
    generated::animation::listener_bool_change_base::ListenerBoolChangeBase,
};
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
    pub fn perform(&self, machine: &mut StateMachineInstance, _invocation: &ListenerInvocation) {
        if self.base.base.nested_input_id() != Core::EMPTY_ID {
            let id = self.base.base.nested_input_id();
            if let Some(current) = machine.nested_bool(id) {
                machine.set_nested_bool(id, self.changed_value(current));
            }
        } else {
            let id = self.base.base.input_id();
            if let Some(current) = machine.bool_input(id).map(|input| input.value()) {
                if let Some(input) = machine.bool_input_mut(id) {
                    input.set_value(self.changed_value(current));
                }
            }
        }
    }
}

impl std::ops::Deref for ListenerBoolChange {
    type Target = ListenerBoolChangeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerBoolChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
