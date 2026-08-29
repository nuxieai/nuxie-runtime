use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeKeyboardImporter {
    listener_input_type_keyboard: CoreHandle,
}

impl ListenerInputTypeKeyboardImporter {
    pub fn new(listener: CoreHandle) -> Self {
        Self {
            listener_input_type_keyboard: listener,
        }
    }

    pub fn listener_input_type_keyboard(&self) -> CoreHandle {
        self.listener_input_type_keyboard.clone()
    }
}

impl ImportStackObject for ListenerInputTypeKeyboardImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
