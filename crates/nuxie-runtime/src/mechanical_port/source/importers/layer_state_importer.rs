use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct LayerStateImporter {
    state: CoreHandle,
}

impl LayerStateImporter {
    pub fn new(state: CoreHandle) -> Self {
        Self { state }
    }

    pub fn add_transition(&mut self, transition: CoreHandle) {
        let added = self
            .state
            .with_mut(|state| state.layer_state_add_transition(transition))
            .expect("LayerStateImporter retains a live state");
        assert!(added, "LayerStateImporter requires a LayerState owner");
    }

    pub fn add_blend_animation(&mut self, animation: CoreHandle) -> bool {
        self.state
            .with_mut(|state| state.blend_state_add_animation(animation))
            .expect("LayerStateImporter retains a live state")
    }
}

impl ImportStackObject for LayerStateImporter {
    fn resolve(&mut self) -> StatusCode {
        let Some(animations) = self
            .state
            .with(|state| state.blend_state_animations())
            .expect("LayerStateImporter retains a live state")
        else {
            return StatusCode::Ok;
        };
        let transition_count = self
            .state
            .with(|state| state.layer_state_transition_count())
            .expect("LayerStateImporter retains a live state")
            .expect("a BlendState remains LayerState-derived");
        for index in 0..transition_count {
            let transition = self
                .state
                .with(|state| state.layer_state_transition(index))
                .expect("LayerStateImporter retains a live state")
                .expect("LayerState transition indices remain valid");
            let Some(exit_id) = transition
                .with(|transition| transition.blend_state_transition_exit_id())
                .expect("LayerState retains live transitions")
            else {
                continue;
            };
            if let Some(animation) = animations.get(exit_id as usize) {
                let assigned = transition
                    .with_mut(|transition| {
                        transition.blend_state_transition_set_exit_animation(animation.clone())
                    })
                    .expect("LayerState retains live transitions");
                assert!(
                    assigned,
                    "a transition exposing a blend exit id must accept its exit animation"
                );
            }
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
