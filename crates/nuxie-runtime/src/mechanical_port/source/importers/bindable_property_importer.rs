use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::data_bind::bindable_property::BindableProperty;

use super::import_stack::ImportStackObject;

pub struct BindablePropertyImporter {
    bindable_property: Option<NonNull<BindableProperty>>,
}

impl BindablePropertyImporter {
    pub fn new(bindable_property: NonNull<BindableProperty>) -> Self {
        Self {
            bindable_property: Some(bindable_property),
        }
    }

    pub fn bindable_property(&mut self) -> Option<NonNull<BindableProperty>> {
        self.bindable_property.take()
    }
}

impl ImportStackObject for BindablePropertyImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
