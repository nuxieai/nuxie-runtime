use crate::mechanical_port::source::animation::{
    listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
};
use crate::mechanical_port::source::generated::animation::focus_action_clear_base::FocusActionClearBase;

#[derive(Default)]
pub struct FocusActionClear {
    pub base: FocusActionClearBase,
}

impl FocusActionClear {
    pub fn perform(
        &self,
        state_machine_instance: Option<&mut StateMachineInstance>,
        _invocation: &ListenerInvocation,
    ) {
        if let Some(manager) =
            state_machine_instance.and_then(StateMachineInstance::focus_manager_mut)
        {
            manager.clear_focus();
        }
    }
}
