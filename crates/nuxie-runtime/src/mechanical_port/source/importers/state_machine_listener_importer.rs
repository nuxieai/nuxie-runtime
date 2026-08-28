use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct StateMachineListenerImporter {
    listener: CoreHandle,
}

impl StateMachineListenerImporter {
    pub fn new(listener: CoreHandle) -> Self {
        Self { listener }
    }
    pub fn state_machine_listener(&self) -> CoreHandle {
        self.listener.clone()
    }
    pub fn add_action(&mut self, action: CoreHandle) {
        self.listener
            .with_mut(|listener| listener.state_machine_listener_add_action(action))
            .filter(|added| *added)
            .expect("imported listener derives from StateMachineListener");
    }
    pub fn add_listener_input_type(&mut self, input_type: CoreHandle) {
        self.listener
            .with_mut(|listener| {
                listener.state_machine_listener_add_listener_input_type(input_type)
            })
            .filter(|added| *added)
            .expect("imported listener derives from StateMachineListener");
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
