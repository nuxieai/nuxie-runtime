use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeSemanticImporter {
    listener_input_type_semantic: CoreHandle,
}

impl ListenerInputTypeSemanticImporter {
    pub fn new(listener: CoreHandle) -> Self {
        Self {
            listener_input_type_semantic: listener,
        }
    }

    pub fn listener_input_type_semantic(&self) -> CoreHandle {
        self.listener_input_type_semantic.clone()
    }
}

impl ImportStackObject for ListenerInputTypeSemanticImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
