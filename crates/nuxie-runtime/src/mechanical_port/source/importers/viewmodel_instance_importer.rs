use std::any::Any;

use crate::mechanical_port::source::{
    core::CoreHandle,
    status_code::StatusCode,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_value::ViewModelInstanceValue,
    },
};

use super::import_stack::ImportStackObject;

pub struct ViewModelInstanceImporter {
    view_model_instance: CoreHandle,
}

impl ViewModelInstanceImporter {
    pub fn new(instance: CoreHandle) -> Self {
        Self {
            view_model_instance: instance,
        }
    }
    pub fn add_value(&mut self, value: &mut ViewModelInstanceValue) {
        self.view_model_instance
            .with_downcast_mut::<ViewModelInstance, _>(|instance| {
                instance.add_value_borrowed(value)
            })
            .expect("ViewModelInstanceImporter retains a ViewModelInstance");
    }
    pub fn view_model_instance(&self) -> CoreHandle {
        self.view_model_instance.clone()
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
