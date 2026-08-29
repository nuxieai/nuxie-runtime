use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    generated::animation::listener_fire_event_base::ListenerFireEventBase,
};

#[derive(Default)]
pub struct ListenerFireEvent {
    pub base: ListenerFireEventBase,
}

impl ListenerFireEvent {
    pub fn perform(
        &self,
        state_machine_instance: &mut StateMachineInstance,
        _invocation: &ListenerInvocation,
    ) {
        let Some(event) = state_machine_instance.resolve_event(self.base.event_id()) else {
            return;
        };
        state_machine_instance.report_event(event, 0.0);
    }
}

impl std::ops::Deref for ListenerFireEvent {
    type Target = ListenerFireEventBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerFireEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
