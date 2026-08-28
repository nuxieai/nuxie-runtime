use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use crate::mechanical_port::source::{
    core::CoreHandle, viewmodel::data_enum::DataEnum as CoreDataEnum,
};
use core::any::Any;
use std::rc::Rc;
pub trait DataEnum: Any {
    fn value(&self, index: u32) -> String;
}

#[derive(Clone)]
pub enum DataEnumRef {
    Core(CoreHandle),
    Static(Rc<dyn DataEnum>),
}

impl DataEnumRef {
    pub fn value(&self, index: u32) -> String {
        match self {
            Self::Core(data_enum) => data_enum
                .with_downcast::<CoreDataEnum, _>(|data_enum| data_enum.value_by_index(index))
                .unwrap_or_default(),
            Self::Static(data_enum) => data_enum.value(index),
        }
    }
}

pub struct DataValueEnum {
    integer: DataValueInteger,
    data_enum: Option<DataEnumRef>,
}
impl Default for DataValueEnum {
    fn default() -> Self {
        Self {
            integer: DataValueInteger::default(),
            data_enum: None,
        }
    }
}
impl DataValueEnum {
    pub const TYPE_KEY: DataType = DataType::Enum;
    pub fn new(value: u32, data_enum: DataEnumRef) -> Self {
        Self {
            integer: DataValueInteger::new(value),
            data_enum: Some(data_enum),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn data_enum(&self) -> Option<DataEnumRef> {
        self.data_enum.clone()
    }
    pub fn set_data_enum(&mut self, value: DataEnumRef) {
        self.data_enum = Some(value)
    }
}
impl DataValue for DataValueEnum {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::Enum || t == DataType::Integer
    }
}
