use std::any::Any;

use crate::mechanical_port::source::{
    animation::{layer_state::LayerState, state_machine_layer::StateMachineLayer},
    artboard::Artboard,
    core::CoreHandle,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateMachineLayerImporter {
    layer: CoreHandle,
    artboard: CoreHandle,
}

impl StateMachineLayerImporter {
    pub fn new(layer: CoreHandle, artboard: CoreHandle) -> Self {
        Self { layer, artboard }
    }

    pub fn add_state(&mut self, state: CoreHandle) {
        self.layer
            .with_downcast_mut::<StateMachineLayer, _>(|layer| layer.add_state(state))
            .expect("StateMachineLayerImporter retains a live StateMachineLayer");
    }
}

impl ImportStackObject for StateMachineLayerImporter {
    fn resolve(&mut self) -> StatusCode {
        let states = self
            .layer
            .with_downcast::<StateMachineLayer, _>(|layer| layer.states().to_vec())
            .expect("StateMachineLayerImporter retains a live StateMachineLayer");
        let animation_count = self
            .artboard
            .with_downcast::<Artboard, _>(Artboard::animation_count)
            .expect("StateMachineLayerImporter retains a live Artboard");
        for state in &states {
            let animation_id = state
                .with(|state| state.animation_state_animation_id())
                .expect("StateMachineLayer retains live states");
            if let Some(animation_id) = animation_id {
                if (animation_id as usize) < animation_count {
                    let animation = self
                        .artboard
                        .with_downcast::<Artboard, _>(|artboard| {
                            artboard.animation_handle_at(animation_id as usize)
                        })
                        .expect("StateMachineLayerImporter retains a live Artboard");
                    let Some(animation) = animation else {
                        return StatusCode::MissingObject;
                    };
                    let assigned = state
                        .with_mut(|state| state.animation_state_set_animation_handle(animation))
                        .expect("StateMachineLayer retains live states");
                    assert!(
                        assigned,
                        "a state exposing an animation id must accept its animation"
                    );
                }
            }
            let transition_count = state
                .with(|state| state.layer_state_transition_count())
                .expect("StateMachineLayer retains live states")
                .expect("StateMachineLayer entries remain LayerState-derived");
            for transition_index in 0..transition_count {
                let transition = state
                    .with(|state| state.layer_state_transition(transition_index))
                    .expect("StateMachineLayer retains live states")
                    .expect("LayerState transition indices remain valid");
                let state_to_id = transition
                    .with(|transition| transition.state_transition_state_to_id())
                    .expect("LayerState retains live transitions")
                    .expect("LayerState transitions remain StateTransition-derived")
                    as usize;
                if let Some(state_to) = states.get(state_to_id) {
                    let assigned = transition
                        .with_mut(|transition| {
                            transition.state_transition_set_state_to(state_to.clone())
                        })
                        .expect("LayerState retains live transitions");
                    assert!(
                        assigned,
                        "a transition exposing stateToId must accept its target state"
                    );
                } else {
                    return StatusCode::InvalidObject;
                }
            }
        }
        StatusCode::Ok
    }

    fn read_null_object(&mut self) -> bool {
        let state = self
            .layer
            .insert_sibling(LayerState::default())
            .expect("StateMachineLayerImporter retains its graph arena");
        self.add_state(state);
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
