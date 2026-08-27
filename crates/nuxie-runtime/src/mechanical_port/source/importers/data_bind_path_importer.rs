use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::data_bind::data_bind_path::DataBindPath;

use super::import_stack::ImportStackObject;

pub struct DataBindPathImporter {
    data_bind_path: Option<NonNull<DataBindPath>>,
}

impl DataBindPathImporter {
    pub fn new(path: NonNull<DataBindPath>) -> Self {
        Self {
            data_bind_path: Some(path),
        }
    }

    pub fn claim(&mut self) -> Option<NonNull<DataBindPath>> {
        self.data_bind_path.take()
    }
}

impl ImportStackObject for DataBindPathImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
