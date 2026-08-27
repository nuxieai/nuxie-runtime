use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{
        state_machine::StateMachine, state_machine_input::StateMachineInput,
        state_machine_layer::StateMachineLayer, state_machine_listener::StateMachineListener,
    },
    data_bind::data_bind::DataBind,
    scripted::scripted_object::ScriptedObject,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct StateMachineImporter {
    state_machine: NonNull<StateMachine>,
}

impl StateMachineImporter {
    pub fn new(state_machine: NonNull<StateMachine>) -> Self {
        Self { state_machine }
    }
    pub fn state_machine(&self) -> NonNull<StateMachine> {
        self.state_machine
    }
    pub fn add_layer(&mut self, layer: Box<StateMachineLayer>) {
        unsafe { self.state_machine.as_mut().add_layer(layer) };
    }
    pub fn add_input(&mut self, input: Option<Box<StateMachineInput>>) {
        unsafe { self.state_machine.as_mut().add_input(input) };
    }
    pub fn add_listener(&mut self, listener: Box<StateMachineListener>) {
        unsafe { self.state_machine.as_mut().add_listener(listener) };
    }
    pub fn add_data_bind(&mut self, data_bind: Box<DataBind>) {
        unsafe { self.state_machine.as_mut().add_data_bind(data_bind) };
    }
    pub fn add_scripted_object(&mut self, object: NonNull<ScriptedObject>) {
        unsafe { self.state_machine.as_mut().add_scripted_object(object) };
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
