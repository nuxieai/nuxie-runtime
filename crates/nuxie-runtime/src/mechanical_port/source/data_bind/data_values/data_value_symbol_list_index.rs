use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueSymbolListIndex {
    integer: DataValueInteger,
}
impl DataValueSymbolListIndex {
    pub const TYPE_KEY: DataType = DataType::SymbolListIndex;
    pub fn new(value: u32) -> Self {
        Self {
            integer: DataValueInteger::new(value),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
}
impl DataValue for DataValueSymbolListIndex {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::SymbolListIndex
    }
}
