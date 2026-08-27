use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    assets::script_asset::ScriptInput, core::CoreTypeKey, custom_property::CustomProperty,
    scripted::scripted_object::ScriptedObject, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ScriptedObjectImporter {
    scripted_object: NonNull<ScriptedObject>,
}

impl ScriptedObjectImporter {
    pub fn new(object: NonNull<ScriptedObject>) -> Self {
        Self {
            scripted_object: object,
        }
    }

    pub fn add_input(&mut self, value: NonNull<CustomProperty>, type_key: CoreTypeKey) {
        if ScriptInput::from(value.cast(), type_key).is_some() {
            unsafe { self.scripted_object.as_mut().add_property(value) };
        }
    }
}

impl ImportStackObject for ScriptedObjectImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
