use std::any::Any;

use crate::mechanical_port::source::{
    artboard::Artboard, core::CoreHandle, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ArtboardImporter {
    artboard: CoreHandle,
}

impl ArtboardImporter {
    pub fn new(artboard: CoreHandle) -> Self {
        Self { artboard }
    }

    pub fn add_component(&mut self, object: Option<CoreHandle>) {
        self.with_artboard(|artboard| artboard.add_object(object));
    }

    pub fn add_animation(&mut self, animation: CoreHandle) {
        self.with_artboard(|artboard| artboard.add_animation(animation));
    }

    pub fn add_state_machine(&mut self, state_machine: CoreHandle) {
        self.with_artboard(|artboard| artboard.add_state_machine(state_machine));
    }

    pub fn add_data_bind(&mut self, data_bind: CoreHandle) {
        self.with_artboard(|artboard| artboard.add_data_bind(data_bind));
    }

    pub fn artboard(&self) -> CoreHandle {
        self.artboard.clone()
    }

    fn with_artboard<R>(&self, f: impl FnOnce(&mut Artboard) -> R) -> R {
        self.artboard
            .with_downcast_mut::<Artboard, _>(f)
            .expect("ArtboardImporter retains an Artboard")
    }
}

impl ImportStackObject for ArtboardImporter {
    fn resolve(&mut self) -> StatusCode {
        if !self.with_artboard(Artboard::validate_objects) {
            return StatusCode::InvalidObject;
        }
        Artboard::initialize_handle(&self.artboard)
    }

    fn read_null_object(&mut self) -> bool {
        self.add_component(None);
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
