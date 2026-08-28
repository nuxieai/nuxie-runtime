use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    focus_data::FocusData,
    generated::animation::focus_action_target_base::FocusActionTargetBase,
};

#[derive(Default)]
pub struct FocusActionTarget {
    pub base: FocusActionTargetBase,
}

impl FocusActionTarget {
    pub fn perform(
        &self,
        state_machine_instance: &mut StateMachineInstance,
        _invocation: &ListenerInvocation,
    ) {
        let Some(target) = state_machine_instance.resolve_artboard_object(self.base.target_id())
        else {
            return;
        };
        let focus_data = target
            .with(|target| {
                target
                    .as_node()?
                    .children()
                    .iter()
                    .find_map(|child| child.with_downcast::<FocusData, _>(|_| child.clone()))
            })
            .flatten();
        if let Some(focus_data) = focus_data {
            state_machine_instance.set_focus(Some(focus_data));
        }
    }
}
