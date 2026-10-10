use std::any::Any;

use crate::mechanical_port::source::{
    assets::script_asset::ScriptInput,
    core::{CoreHandle, CoreTypeKey},
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct ScriptedObjectImporter {
    scripted_object: CoreHandle,
    input_parent_id: u32,
}

impl ScriptedObjectImporter {
    pub fn new(object: CoreHandle, input_parent_id: u32) -> Self {
        Self {
            scripted_object: object,
            input_parent_id,
        }
    }

    pub fn scripted_object(&self) -> CoreHandle {
        self.scripted_object.clone()
    }

    pub fn input_parent_id(&self) -> u32 {
        self.input_parent_id
    }

    pub fn add_input(&mut self, value: CoreHandle, type_key: CoreTypeKey, input: &mut ScriptInput) {
        if ScriptInput::from(value.clone(), type_key).is_some() {
            self.scripted_object
                .with_mut(|object| object.scripted_object_add_property_from_input(value, input))
                .filter(|added| *added)
                .expect("imported owner derives from ScriptedObject");
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
