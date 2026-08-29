use crate::mechanical_port::source::{
    animation::{
        state_instance::{StateInstance, StateInstanceBehavior},
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub struct SystemStateInstance {
    pub base: StateInstance,
}

impl StateInstanceBehavior for SystemStateInstance {
    fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance) {
        Self::advance(self, seconds, state_machine_instance);
    }

    fn apply(&mut self, artboard_instance: &RuntimeArtboardInstanceWeakHandle, mix: f32) {
        Self::apply(self, artboard_instance, mix);
    }

    fn keep_going(&self) -> bool {
        Self::keep_going(self)
    }
}

impl SystemStateInstance {
    pub fn new(layer_state: CoreHandle) -> Self {
        Self {
            base: StateInstance::new(layer_state),
        }
    }

    pub fn advance(&mut self, _seconds: f32, _state_machine_instance: &mut StateMachineInstance) {}

    pub fn apply(&mut self, _artboard: &RuntimeArtboardInstanceWeakHandle, _mix: f32) {}

    pub fn keep_going(&self) -> bool {
        false
    }
}
