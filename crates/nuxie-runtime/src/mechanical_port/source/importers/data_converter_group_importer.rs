use std::any::Any;

use crate::mechanical_port::source::core::CoreHandle;

use super::import_stack::ImportStackObject;

pub struct DataConverterGroupImporter {
    group: CoreHandle,
}

impl DataConverterGroupImporter {
    pub fn new(group: CoreHandle) -> Self {
        Self { group }
    }

    pub fn group(&self) -> CoreHandle {
        self.group.clone()
    }
}

impl ImportStackObject for DataConverterGroupImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
