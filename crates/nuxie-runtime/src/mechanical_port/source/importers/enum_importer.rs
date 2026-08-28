use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct EnumImporter {
    data_enum: CoreHandle,
}

impl EnumImporter {
    pub fn new(data_enum: CoreHandle) -> Self {
        Self { data_enum }
    }

    pub fn add_value(&mut self, value: CoreHandle) {
        self.data_enum
            .with_mut(|data_enum| {
                data_enum
                    .as_data_enum_mut()
                    .expect("imported enum derives from DataEnum")
                    .add_value(value)
            })
            .expect("EnumImporter retains a live enum");
    }

    pub fn data_enum(&self) -> CoreHandle {
        self.data_enum.clone()
    }
}

impl ImportStackObject for EnumImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
