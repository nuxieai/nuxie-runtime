use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub struct StateInstance {
    layer_state: CoreHandle,
}

impl StateInstance {
    pub fn new(layer_state: CoreHandle) -> Self {
        Self { layer_state }
    }

    pub fn clear_spilled_time(&mut self) {}

    pub fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }

    pub fn state(&self) -> CoreHandle {
        self.layer_state.clone()
    }
}

impl Drop for StateInstance {
    fn drop(&mut self) {}
}

pub trait StateInstanceBehavior {
    fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance);
    fn apply(&mut self, artboard_instance: &RuntimeArtboardInstanceWeakHandle, mix: f32);
    fn keep_going(&self) -> bool;
    fn clear_spilled_time(&mut self) {}
    fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }
}
