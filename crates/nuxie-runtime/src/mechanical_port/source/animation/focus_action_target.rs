use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    generated::animation::focus_action_target_base::FocusActionTargetBase,
};

#[derive(Clone, Copy)]
pub enum FocusTargetChild {
    FocusData(*mut ()),
    Other,
}

pub trait FocusActionStateMachine {
    fn resolved_node_children(&mut self, target_id: u32) -> Option<Vec<FocusTargetChild>>;
    fn set_focus(&mut self, focus_data: *mut ());
}

#[derive(Default)]
pub struct FocusActionTarget {
    pub base: FocusActionTargetBase,
}

impl FocusActionTarget {
    pub fn perform(
        &self,
        state_machine_instance: &mut dyn FocusActionStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        let Some(children) = state_machine_instance.resolved_node_children(self.base.target_id())
        else {
            return;
        };

        let mut focus_data = None;
        for child in children {
            if let FocusTargetChild::FocusData(value) = child {
                focus_data = Some(value);
                break;
            }
        }
        if let Some(focus_data) = focus_data {
            state_machine_instance.set_focus(focus_data);
        }
    }
}
