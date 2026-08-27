use crate::mechanical_port::source::animation::{
    layer_state::LayerState, linear_animation_instance::LinearAnimationInstance,
};

pub struct StateInstance {
    layer_state: *const LayerState,
}

impl StateInstance {
    pub fn new(layer_state: &LayerState) -> Self {
        Self { layer_state }
    }

    pub fn clear_spilled_time(&mut self) {}

    pub fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }

    pub fn state(&self) -> &LayerState {
        unsafe { &*self.layer_state }
    }
}

impl Drop for StateInstance {
    fn drop(&mut self) {}
}

pub trait StateInstanceBehavior {
    fn advance(&mut self, seconds: f32, state_machine_instance: *mut ());
    fn apply(&mut self, artboard_instance: *mut (), mix: f32);
    fn keep_going(&self) -> bool;
    fn clear_spilled_time(&mut self) {}
    fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }
}
