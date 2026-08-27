use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type_gamepad::ListenerInputTypeGamepad,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeGamepadImporter {
    listener_input_type_gamepad: NonNull<ListenerInputTypeGamepad>,
}

impl ListenerInputTypeGamepadImporter {
    pub fn new(listener: NonNull<ListenerInputTypeGamepad>) -> Self {
        Self {
            listener_input_type_gamepad: listener,
        }
    }

    pub fn listener_input_type_gamepad(&self) -> NonNull<ListenerInputTypeGamepad> {
        self.listener_input_type_gamepad
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
