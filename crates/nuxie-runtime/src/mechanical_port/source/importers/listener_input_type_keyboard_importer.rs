use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ListenerInputTypeKeyboardImporter {
    listener_input_type_keyboard: NonNull<ListenerInputTypeKeyboard>,
}

impl ListenerInputTypeKeyboardImporter {
    pub fn new(listener: NonNull<ListenerInputTypeKeyboard>) -> Self {
        Self {
            listener_input_type_keyboard: listener,
        }
    }

    pub fn listener_input_type_keyboard(&self) -> NonNull<ListenerInputTypeKeyboard> {
        self.listener_input_type_keyboard
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
