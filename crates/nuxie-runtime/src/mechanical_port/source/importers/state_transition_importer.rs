use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{state_transition::StateTransition, transition_condition::TransitionCondition},
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateTransitionImporter {
    transition: NonNull<StateTransition>,
}

impl StateTransitionImporter {
    pub fn new(transition: NonNull<StateTransition>) -> Self {
        Self { transition }
    }

    pub fn add_condition(&mut self, condition: NonNull<TransitionCondition>) {
        unsafe { self.transition.as_mut().add_condition(condition) };
    }
}

impl ImportStackObject for StateTransitionImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
