use std::ptr::NonNull;

use crate::mechanical_port::source::{
    event::Event, generated::animation::state_machine_fire_event_base::StateMachineFireEventBase,
};

#[derive(Clone, Copy)]
pub enum ResolvedFireEvent {
    Event(NonNull<Event>),
    Other,
}

pub trait FireEventStateMachine {
    fn resolve_core_event(&mut self, event_id: u32) -> Option<ResolvedFireEvent>;
    fn report_event(&mut self, event: NonNull<Event>);
}

#[derive(Default)]
pub struct StateMachineFireEvent {
    pub base: StateMachineFireEventBase,
}

impl StateMachineFireEvent {
    pub fn perform(&self, state_machine_instance: &mut dyn FireEventStateMachine) {
        let Some(ResolvedFireEvent::Event(event)) =
            state_machine_instance.resolve_core_event(self.base.event_id())
        else {
            return;
        };
        state_machine_instance.report_event(event);
    }
}
