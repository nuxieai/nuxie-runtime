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
        if let Some(machine) = state_machine_instance {
            machine
                .focus_manager()
                .with_focus_manager_mut(|manager| manager.clear_focus());
        }
    }
}

impl std::ops::Deref for FocusActionClear {
    type Target = FocusActionClearBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FocusActionClear {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
