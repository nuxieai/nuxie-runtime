use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    generated::animation::focus_action_traversal_base::FocusActionTraversalBase,
};

pub trait FocusTraversalManager {
    fn focus_next(&mut self);
    fn focus_previous(&mut self);
    fn focus_up(&mut self);
    fn focus_down(&mut self);
    fn focus_left(&mut self);
    fn focus_right(&mut self);
}

#[derive(Default)]
pub struct FocusActionTraversal {
    pub base: FocusActionTraversalBase,
}

impl FocusActionTraversal {
    pub fn perform(
        &self,
        state_machine_instance: Option<&mut dyn FocusTraversalManager>,
        _invocation: &ListenerInvocation,
    ) {
        let Some(manager) = state_machine_instance else {
            return;
        };
        match self.base.traversal_kind() {
            1 => manager.focus_previous(),
            2 => manager.focus_up(),
            3 => manager.focus_down(),
            4 => manager.focus_left(),
            5 => manager.focus_right(),
            _ => manager.focus_next(),
        }
    }
}
