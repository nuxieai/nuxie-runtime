use std::any::Any;

use crate::mechanical_port::source::{animation::keyed_object::KeyedObject, core::CoreHandle};

use super::import_stack::ImportStackObject;

pub struct KeyedObjectImporter {
    keyed_object: CoreHandle,
}

impl KeyedObjectImporter {
    pub fn new(keyed_object: CoreHandle) -> Self {
        Self { keyed_object }
    }

    pub fn add_keyed_property(&mut self, property: CoreHandle) {
        self.keyed_object
            .with_downcast_mut::<KeyedObject, _>(|keyed_object| {
                keyed_object.add_keyed_property(property)
            })
            .expect("KeyedObjectImporter retains a KeyedObject");
    }
}

impl ImportStackObject for KeyedObjectImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
