use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
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
                    .find(|child| child.is_type_of(crate::mechanical_port::source::generated::focus_data_base::FocusDataBase::TYPE_KEY))
                    .cloned()
            })
            .flatten();
        if let Some(focus_data) = focus_data {
            state_machine_instance.set_focus(Some(focus_data));
        }
    }
}

impl std::ops::Deref for FocusActionTarget {
    type Target = FocusActionTargetBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FocusActionTarget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
