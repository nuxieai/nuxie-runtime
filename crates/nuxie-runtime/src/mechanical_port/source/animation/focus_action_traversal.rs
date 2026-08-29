use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    generated::animation::focus_action_traversal_base::FocusActionTraversalBase,
};

#[derive(Default)]
pub struct FocusActionTraversal {
    pub base: FocusActionTraversalBase,
}

impl FocusActionTraversal {
    pub fn perform(
        &self,
        state_machine_instance: Option<&mut StateMachineInstance>,
        _invocation: &ListenerInvocation,
    ) {
        let Some(manager) = state_machine_instance else {
            return;
        };
        match self.base.traversal_kind() {
            1 => {
                manager.focus_previous();
            }
            2 => {
                manager.focus_up();
            }
            3 => {
                manager.focus_down();
            }
            4 => {
                manager.focus_left();
            }
            5 => {
                manager.focus_right();
            }
            _ => {
                manager.focus_next();
            }
        }
    }
}

impl std::ops::Deref for FocusActionTraversal {
    type Target = FocusActionTraversalBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FocusActionTraversal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
