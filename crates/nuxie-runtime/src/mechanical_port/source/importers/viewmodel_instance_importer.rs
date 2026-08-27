use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    status_code::StatusCode,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_value::ViewModelInstanceValue,
    },
};

use super::import_stack::ImportStackObject;

pub struct ViewModelInstanceImporter {
    view_model_instance: NonNull<ViewModelInstance>,
}

impl ViewModelInstanceImporter {
    pub fn new(instance: NonNull<ViewModelInstance>) -> Self {
        Self {
            view_model_instance: instance,
        }
    }
    pub fn add_value(&mut self, value: NonNull<ViewModelInstanceValue>) {
        unsafe { self.view_model_instance.as_mut().add_value(value) };
    }
    pub fn view_model_instance(&self) -> NonNull<ViewModelInstance> {
        self.view_model_instance
    }
}

impl ImportStackObject for ViewModelInstanceImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
