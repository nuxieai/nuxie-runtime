use std::any::Any;

use crate::mechanical_port::source::{
    core::CoreHandle, status_code::StatusCode, viewmodel::viewmodel::ViewModel,
};

use super::import_stack::ImportStackObject;

pub struct ViewModelImporter {
    view_model: CoreHandle,
}

impl ViewModelImporter {
    pub fn new(view_model: CoreHandle) -> Self {
        Self { view_model }
    }
    pub fn add_property(&mut self, property: CoreHandle) {
        self.view_model
            .with_downcast_mut::<ViewModel, _>(|view_model| view_model.add_property(property))
            .expect("ViewModelImporter retains a ViewModel");
    }
}

impl ImportStackObject for ViewModelImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
