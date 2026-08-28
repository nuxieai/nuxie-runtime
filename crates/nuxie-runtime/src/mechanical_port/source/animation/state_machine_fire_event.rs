use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::animation::state_machine_fire_event_base::StateMachineFireEventBase,
};

#[derive(Clone)]
pub enum ResolvedFireEvent {
    Event(CoreHandle),
    Other,
}

pub trait FireEventStateMachine {
    fn resolve_core_event(&mut self, event_id: u32) -> Option<ResolvedFireEvent>;
    fn report_event(&mut self, event: CoreHandle);
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
