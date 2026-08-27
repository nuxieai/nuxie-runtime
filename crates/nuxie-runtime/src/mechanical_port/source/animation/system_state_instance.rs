use crate::mechanical_port::source::animation::{
    layer_state::LayerState,
    state_instance::{StateInstance, StateInstanceBehavior},
};

pub struct SystemStateInstance {
    pub base: StateInstance,
}

impl StateInstanceBehavior for SystemStateInstance {
    fn advance(&mut self, seconds: f32, state_machine_instance: *mut ()) {
        Self::advance(self, seconds, state_machine_instance);
    }

    fn apply(&mut self, artboard_instance: *mut (), mix: f32) {
        Self::apply(self, artboard_instance, mix);
    }

    fn keep_going(&self) -> bool {
        Self::keep_going(self)
    }
}

impl SystemStateInstance {
    pub fn new(layer_state: &LayerState, _instance: *mut ()) -> Self {
        Self {
            base: StateInstance::new(layer_state),
        }
    }

    pub fn advance(&mut self, _seconds: f32, _state_machine_instance: *mut ()) {}

    pub fn apply(&mut self, _artboard: *mut (), _mix: f32) {}

    pub fn keep_going(&self) -> bool {
        false
    }
}
