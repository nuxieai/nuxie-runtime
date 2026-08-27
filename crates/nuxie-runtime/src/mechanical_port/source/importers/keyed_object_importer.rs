use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::animation::{
    keyed_object::KeyedObject, keyed_property::KeyedProperty,
};

use super::import_stack::ImportStackObject;

pub struct KeyedObjectImporter {
    keyed_object: NonNull<KeyedObject>,
}

impl KeyedObjectImporter {
    pub fn new(keyed_object: NonNull<KeyedObject>) -> Self {
        Self { keyed_object }
    }

    pub fn add_keyed_property(&mut self, property: Box<KeyedProperty>) {
        unsafe { self.keyed_object.as_mut().add_keyed_property(property) };
    }
}

impl ImportStackObject for KeyedObjectImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
