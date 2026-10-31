use std::any::Any;

use crate::mechanical_port::source::{
    animation::{layer_state::LayerState, state_machine_layer::StateMachineLayer},
    artboard::Artboard,
    core::CoreHandle,
    generated::animation::{
        any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
        exit_state_base::ExitStateBase,
    },
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

    /// Nuxie state holds (docs/live-moves.md, State holds): a transition with
    /// `isHold` (60023) must lead from an authored state to itself, leave exit
    /// time off and carry a condition; one that breaks a rule refuses the
    /// file. An accepted hold is registered on the layer.
    fn resolve_state_hold(
        &mut self,
        state_index: usize,
        state: &CoreHandle,
        transition: &CoreHandle,
        state_to_id: usize,
    ) -> StatusCode {
        let Some((enables_exit_time, condition_count)) = transition
            .with(|transition| {
                transition
                    .as_state_transition()
                    .filter(|transition| transition.is_hold())
                    .map(|transition| (transition.enable_exit_time(), transition.condition_count()))
            })
            .expect("LayerState retains live transitions")
        else {
            return StatusCode::Ok;
        };
        let rule = if state_to_id != state_index {
            Some("its target is another state")
        } else if matches!(
            state.core_type(),
            Some(AnyStateBase::TYPE_KEY | EntryStateBase::TYPE_KEY | ExitStateBase::TYPE_KEY)
        ) {
            Some("it is on an Any, Entry or Exit state")
        } else if enables_exit_time {
            Some("it enables exit time")
        } else if condition_count == 0 {
            Some("it has no condition")
        } else {
            None
        };
        if let Some(rule) = rule {
            let layer_name = self
                .layer
                .with_downcast::<StateMachineLayer, _>(|layer| layer.name().to_owned())
                .expect("StateMachineLayerImporter retains a live StateMachineLayer");
            eprintln!("Refused state hold in layer {layer_name}: {rule}");
            return StatusCode::InvalidObject;
        }
        self.layer
            .with_downcast_mut::<StateMachineLayer, _>(|layer| {
                layer.add_state_hold(state.clone(), transition.clone())
            })
            .expect("StateMachineLayerImporter retains a live StateMachineLayer");
        StatusCode::Ok
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
        for (state_index, state) in states.iter().enumerate() {
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
                let code = self.resolve_state_hold(state_index, state, &transition, state_to_id);
                if code != StatusCode::Ok {
                    return code;
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
