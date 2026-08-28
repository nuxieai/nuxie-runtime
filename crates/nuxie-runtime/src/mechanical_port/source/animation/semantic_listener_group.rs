use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeObjectHandle, RuntimeServicesHandle, RuntimeStateMachineInstanceWeakHandle,
    },
    core::CoreHandle,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SemanticActionType {
    Tap = 0,
    Increase = 1,
    Decrease = 2,
}

impl SemanticActionType {
    pub const fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Tap,
            1 => Self::Increase,
            2 => Self::Decrease,
            _ => return None,
        })
    }
}

pub struct SemanticListenerGroup {
    runtime: RuntimeServicesHandle,
    semantic_data: CoreHandle,
    listener: CoreHandle,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    group: RuntimeObjectHandle,
}

impl SemanticListenerGroup {
    pub fn new(
        runtime: RuntimeServicesHandle,
        semantic_data: CoreHandle,
        listener: CoreHandle,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Box<Self> {
        let group = runtime.borrow_mut().semantic_data_add_listener(
            &semantic_data,
            &listener,
            state_machine_instance.clone(),
        );
        Box::new(Self {
            runtime,
            semantic_data,
            listener,
            state_machine_instance,
            group,
        })
    }

    pub fn listener(&self) -> CoreHandle {
        self.listener.clone()
    }

    pub fn semantic_data(&self) -> CoreHandle {
        self.semantic_data.clone()
    }

    fn queue_if_listening(&mut self, action: SemanticActionType) {
        self.state_machine_instance.with_instance_mut(|machine| {
            if machine.semantic_constraints_met(&self.listener, action) {
                machine.queue_semantic_event(self.group, action as u8);
            }
        });
    }

    pub fn on_semantic_tap(&mut self) {
        self.queue_if_listening(SemanticActionType::Tap);
    }

    pub fn on_semantic_increase(&mut self) {
        self.queue_if_listening(SemanticActionType::Increase);
    }

    pub fn on_semantic_decrease(&mut self) {
        self.queue_if_listening(SemanticActionType::Decrease);
    }
}

impl Drop for SemanticListenerGroup {
    fn drop(&mut self) {
        self.runtime
            .borrow_mut()
            .semantic_data_remove_listener(&self.semantic_data, self.group);
    }
}
