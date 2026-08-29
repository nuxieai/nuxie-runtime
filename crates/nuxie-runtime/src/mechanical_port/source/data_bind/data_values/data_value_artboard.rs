use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use core::any::Any;
#[derive(Clone, Debug)]
pub struct DataValueArtboard {
    integer: DataValueInteger,
}
impl Default for DataValueArtboard {
    fn default() -> Self {
        Self::new(Self::DEFAULT_VALUE)
    }
}
impl DataValueArtboard {
    pub const TYPE_KEY: DataType = DataType::Artboard;
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn new(value: u32) -> Self {
        Self {
            integer: DataValueInteger::new(value),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn set_value(&mut self, value: u32) {
        self.integer.set_value(value)
    }
}
impl DataValue for DataValueArtboard {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::Artboard || t == DataType::Integer
    }
}
