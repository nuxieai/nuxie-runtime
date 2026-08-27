use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{layer_state::LayerState, state_machine_layer::StateMachineLayer},
    artboard::Artboard,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateMachineLayerImporter {
    layer: NonNull<StateMachineLayer>,
    artboard: NonNull<Artboard>,
}

impl StateMachineLayerImporter {
    pub fn new(layer: NonNull<StateMachineLayer>, artboard: NonNull<Artboard>) -> Self {
        Self { layer, artboard }
    }

    pub fn add_state(&mut self, state: Box<LayerState>) {
        unsafe { self.layer.as_mut().add_state(state) };
    }
}

impl ImportStackObject for StateMachineLayerImporter {
    fn resolve(&mut self) -> StatusCode {
        let layer = unsafe { self.layer.as_mut() };
        let artboard = unsafe { self.artboard.as_ref() };
        for state in layer.states_mut() {
            if let Some(animation_state) = state.as_animation_state_mut() {
                let animation_id = animation_state.animation_id() as usize;
                if animation_id < artboard.animation_count() {
                    let animation = artboard.animation(animation_id);
                    animation_state.set_animation(animation);
                    if animation_state.animation().is_none() {
                        return StatusCode::MissingObject;
                    }
                }
            }
            for transition in state.transitions_mut() {
                let state_to_id = transition.state_to_id() as usize;
                if state_to_id < layer.states().len() {
                    transition.set_state_to(layer.state(state_to_id));
                } else {
                    return StatusCode::InvalidObject;
                }
            }
        }
        StatusCode::Ok
    }

    fn read_null_object(&mut self) -> bool {
        self.add_state(Box::new(LayerState::default()));
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
