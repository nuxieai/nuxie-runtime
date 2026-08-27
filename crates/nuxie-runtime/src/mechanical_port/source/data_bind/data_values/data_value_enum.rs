use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use core::any::Any;
pub trait DataEnum: Any {
    fn value(&self, index: u32) -> String;
}
pub struct DataValueEnum {
    integer: DataValueInteger,
    data_enum: Option<*mut dyn DataEnum>,
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
    pub fn new(value: u32, data_enum: *mut dyn DataEnum) -> Self {
        Self {
            integer: DataValueInteger::new(value),
            data_enum: Some(data_enum),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn data_enum(&self) -> Option<*mut dyn DataEnum> {
        self.data_enum
    }
    pub fn set_data_enum(&mut self, value: *mut dyn DataEnum) {
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
