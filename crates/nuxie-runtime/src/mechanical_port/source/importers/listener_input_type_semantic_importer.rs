use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type_semantic::ListenerInputTypeSemantic,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeSemanticImporter {
    listener_input_type_semantic: NonNull<ListenerInputTypeSemantic>,
}

impl ListenerInputTypeSemanticImporter {
    pub fn new(listener: NonNull<ListenerInputTypeSemantic>) -> Self {
        Self {
            listener_input_type_semantic: listener,
        }
    }

    pub fn listener_input_type_semantic(&self) -> NonNull<ListenerInputTypeSemantic> {
        self.listener_input_type_semantic
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
