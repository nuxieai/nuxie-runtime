use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueInteger {
    value: u32,
}
impl DataValueInteger {
    pub const TYPE_KEY: DataType = DataType::Integer;
    pub const DEFAULT_VALUE: u32 = 0;
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn set_value(&mut self, value: u32) {
        self.value = value
    }
}
impl DataValue for DataValueInteger {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::Integer
    }
}
