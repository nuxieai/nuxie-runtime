use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueBoolean {
    value: bool,
}
impl DataValueBoolean {
    pub const TYPE_KEY: DataType = DataType::Boolean;
    pub const DEFAULT_VALUE: bool = false;
    pub fn new(value: bool) -> Self {
        Self { value }
    }
    pub fn value(&self) -> bool {
        self.value
    }
    pub fn set_value(&mut self, value: bool) {
        self.value = value
    }
}
impl DataValue for DataValueBoolean {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::Boolean
    }
}
