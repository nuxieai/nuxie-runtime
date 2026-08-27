use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::data_bind::converters::data_converter_group::DataConverterGroup;

use super::import_stack::ImportStackObject;

pub struct DataConverterGroupImporter {
    group: NonNull<DataConverterGroup>,
}

impl DataConverterGroupImporter {
    pub fn new(group: NonNull<DataConverterGroup>) -> Self {
        Self { group }
    }

    pub fn group(&self) -> NonNull<DataConverterGroup> {
        self.group
    }
}

impl ImportStackObject for DataConverterGroupImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
