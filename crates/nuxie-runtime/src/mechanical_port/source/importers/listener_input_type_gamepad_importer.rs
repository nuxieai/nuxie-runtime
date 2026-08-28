use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeGamepadImporter {
    listener_input_type_gamepad: CoreHandle,
}

impl ListenerInputTypeGamepadImporter {
    pub fn new(listener: CoreHandle) -> Self {
        Self {
            listener_input_type_gamepad: listener,
        }
    }

    pub fn listener_input_type_gamepad(&self) -> CoreHandle {
        self.listener_input_type_gamepad.clone()
    }
}

impl ImportStackObject for ListenerInputTypeGamepadImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
