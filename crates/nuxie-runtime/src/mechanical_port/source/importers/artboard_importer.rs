use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{linear_animation::LinearAnimation, state_machine::StateMachine},
    artboard::Artboard,
    core::Core,
    data_bind::data_bind::DataBind,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ArtboardImporter {
    artboard: NonNull<Artboard>,
}

impl ArtboardImporter {
    pub fn new(artboard: NonNull<Artboard>) -> Self {
        Self { artboard }
    }

    pub fn add_component(&mut self, object: Option<NonNull<Core>>) {
        unsafe { self.artboard.as_mut().add_object(object) };
    }

    pub fn add_animation(&mut self, animation: NonNull<LinearAnimation>) {
        unsafe { self.artboard.as_mut().add_animation(animation) };
    }

    pub fn add_state_machine(&mut self, state_machine: NonNull<StateMachine>) {
        unsafe { self.artboard.as_mut().add_state_machine(state_machine) };
    }

    pub fn add_data_bind(&mut self, data_bind: NonNull<DataBind>) {
        unsafe { self.artboard.as_mut().add_data_bind(data_bind) };
    }

    pub fn artboard(&self) -> NonNull<Artboard> {
        self.artboard
    }
}

impl ImportStackObject for ArtboardImporter {
    fn resolve(&mut self) -> StatusCode {
        let artboard = unsafe { self.artboard.as_mut() };
        if !artboard.validate_objects() {
            return StatusCode::InvalidObject;
        }
        artboard.initialize()
    }

    fn read_null_object(&mut self) -> bool {
        self.add_component(None);
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
