use crate::mechanical_port::source::{
    animation::state_machine_instance::StateMachineInstance,
    generated::animation::state_machine_fire_event_base::StateMachineFireEventBase,
};

#[derive(Default)]
pub struct StateMachineFireEvent {
    pub base: StateMachineFireEventBase,
}

impl StateMachineFireEvent {
    pub fn perform(&self, state_machine_instance: &mut StateMachineInstance) {
        let Some(event) = state_machine_instance.resolve_event(self.base.event_id()) else {
            return;
        };
        state_machine_instance.report_event(event, 0.0);
    }
}
