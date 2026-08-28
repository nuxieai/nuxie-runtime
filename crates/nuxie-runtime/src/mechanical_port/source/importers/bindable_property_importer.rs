use std::any::Any;

use crate::mechanical_port::source::core::CoreHandle;

use super::import_stack::ImportStackObject;

pub struct BindablePropertyImporter {
    bindable_property: Option<CoreHandle>,
}

impl BindablePropertyImporter {
    pub fn new(bindable_property: CoreHandle) -> Self {
        Self {
            bindable_property: Some(bindable_property),
        }
    }

    pub fn bindable_property(&mut self) -> Option<CoreHandle> {
        self.bindable_property.take()
    }
}

impl ImportStackObject for BindablePropertyImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
