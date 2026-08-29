use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueString {
    value: String,
}
impl DataValueString {
    pub const TYPE_KEY: DataType = DataType::String;
    pub const DEFAULT_VALUE: &'static str = "";
    pub fn new(value: String) -> Self {
        Self { value }
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, value: String) {
        self.value = value
    }
}
impl DataValue for DataValueString {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::String
    }
}
