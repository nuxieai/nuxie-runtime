use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    status_code::StatusCode,
    viewmodel::{data_enum::DataEnum, data_enum_value::DataEnumValue},
};

use super::import_stack::ImportStackObject;

pub struct EnumImporter {
    data_enum: NonNull<DataEnum>,
}

impl EnumImporter {
    pub fn new(data_enum: NonNull<DataEnum>) -> Self {
        Self { data_enum }
    }

    pub fn add_value(&mut self, value: NonNull<DataEnumValue>) {
        unsafe { self.data_enum.as_mut().add_value(value) };
    }

    pub fn data_enum(&self) -> NonNull<DataEnum> {
        self.data_enum
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
