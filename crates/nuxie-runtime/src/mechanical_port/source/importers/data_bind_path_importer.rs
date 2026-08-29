use std::any::Any;

use crate::mechanical_port::source::core::CoreHandle;

use super::import_stack::ImportStackObject;

pub struct DataBindPathImporter {
    data_bind_path: Option<CoreHandle>,
}

impl DataBindPathImporter {
    pub fn new(path: CoreHandle) -> Self {
        Self {
            data_bind_path: Some(path),
        }
    }

    pub fn claim(&mut self) -> Option<CoreHandle> {
        self.data_bind_path.take()
    }
}

impl ImportStackObject for DataBindPathImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
