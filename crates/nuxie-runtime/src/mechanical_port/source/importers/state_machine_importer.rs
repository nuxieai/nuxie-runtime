use std::any::Any;

use crate::mechanical_port::source::{
    animation::state_machine::StateMachine, core::CoreHandle, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateMachineImporter {
    state_machine: CoreHandle,
}

impl StateMachineImporter {
    pub fn new(state_machine: CoreHandle) -> Self {
        Self { state_machine }
    }
    pub fn state_machine(&self) -> CoreHandle {
        self.state_machine.clone()
    }
    pub fn add_layer(&mut self, layer: CoreHandle) {
        self.with_state_machine(|state_machine| state_machine.add_layer(layer));
    }
    pub fn add_input(&mut self, input: Option<CoreHandle>) {
        self.with_state_machine(|state_machine| state_machine.add_input(input));
    }
    pub fn add_listener(&mut self, listener: CoreHandle) {
        self.with_state_machine(|state_machine| state_machine.add_listener(listener));
    }
    pub fn add_data_bind(&mut self, data_bind: CoreHandle) {
        self.with_state_machine(|state_machine| state_machine.add_data_bind(data_bind));
    }
    pub fn add_scripted_object(&mut self, object: CoreHandle) {
        self.with_state_machine(|state_machine| state_machine.add_scripted_object(object));
    }

    fn with_state_machine(&self, f: impl FnOnce(&mut StateMachine)) {
        self.state_machine
            .with_downcast_mut::<StateMachine, _>(f)
            .expect("StateMachineImporter retains a StateMachine");
    }
}

impl ImportStackObject for StateMachineImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn read_null_object(&mut self) -> bool {
        self.add_input(None);
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
