use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{
        listener_action::ListenerAction, listener_types::listener_input_type::ListenerInputType,
        state_machine_listener::StateMachineListener,
    },
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateMachineListenerImporter {
    listener: NonNull<StateMachineListener>,
}

impl StateMachineListenerImporter {
    pub fn new(listener: NonNull<StateMachineListener>) -> Self {
        Self { listener }
    }
    pub fn state_machine_listener(&self) -> NonNull<StateMachineListener> {
        self.listener
    }
    pub fn add_action(&mut self, action: Box<ListenerAction>) {
        unsafe { self.listener.as_mut().add_action(action) };
    }
    pub fn add_listener_input_type(&mut self, input_type: Box<ListenerInputType>) {
        unsafe { self.listener.as_mut().add_listener_input_type(input_type) };
    }
}

impl ImportStackObject for StateMachineListenerImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
