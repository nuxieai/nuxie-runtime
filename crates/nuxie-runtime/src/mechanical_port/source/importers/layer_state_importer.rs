use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{
        blend_animation::BlendAnimation, layer_state::LayerState, state_transition::StateTransition,
    },
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct LayerStateImporter {
    state: NonNull<LayerState>,
}

impl LayerStateImporter {
    pub fn new(state: NonNull<LayerState>) -> Self {
        Self { state }
    }

    pub fn add_transition(&mut self, transition: NonNull<StateTransition>) {
        unsafe { self.state.as_mut().add_transition(transition) };
    }

    pub fn add_blend_animation(&mut self, animation: NonNull<BlendAnimation>) -> bool {
        let state = unsafe { self.state.as_mut() };
        let Some(blend_state) = state.as_blend_state_mut() else {
            return false;
        };
        blend_state.add_animation(animation);
        true
    }
}

impl ImportStackObject for LayerStateImporter {
    fn resolve(&mut self) -> StatusCode {
        let state = unsafe { self.state.as_mut() };
        if let Some(blend_state) = state.as_blend_state_mut() {
            let transitions = blend_state.transitions().to_vec();
            for transition in transitions {
                let transition = unsafe { transition.as_mut() };
                let Some(blend_transition) = transition.as_blend_state_transition_mut() else {
                    continue;
                };
                let exit_id = blend_transition.exit_blend_animation_id() as usize;
                if let Some(animation) = blend_state.animations().get(exit_id).copied() {
                    blend_transition.set_exit_blend_animation(animation);
                }
            }
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
