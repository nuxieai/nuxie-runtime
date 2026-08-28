use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct StateTransitionImporter {
    transition: CoreHandle,
}

impl StateTransitionImporter {
    pub fn new(transition: CoreHandle) -> Self {
        Self { transition }
    }

    pub fn add_condition(&mut self, condition: CoreHandle) {
        self.transition
            .with_mut(|transition| transition.state_transition_add_condition(condition))
            .filter(|added| *added)
            .expect("imported transition derives from StateTransition");
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
