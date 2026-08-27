use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_fire_event::FireEventStateMachine,
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
        state_machine_instance: &mut dyn FireEventStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        let Some(
            crate::mechanical_port::source::animation::state_machine_fire_event::ResolvedFireEvent::Event(event),
        ) = state_machine_instance.resolve_core_event(self.base.event_id())
        else {
            return;
        };
        state_machine_instance.report_event(event);
    }
}
