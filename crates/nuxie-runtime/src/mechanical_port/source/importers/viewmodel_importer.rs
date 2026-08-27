use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    status_code::StatusCode,
    viewmodel::{viewmodel::ViewModel, viewmodel_property::ViewModelProperty},
};

use super::import_stack::ImportStackObject;

pub struct ViewModelImporter {
    view_model: NonNull<ViewModel>,
}

impl ViewModelImporter {
    pub fn new(view_model: NonNull<ViewModel>) -> Self {
        Self { view_model }
    }
    pub fn add_property(&mut self, property: NonNull<ViewModelProperty>) {
        unsafe { self.view_model.as_mut().add_property(property) };
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
