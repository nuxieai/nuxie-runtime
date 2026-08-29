use super::{data_type::DataType, data_value::DataValue};
use crate::mechanical_port::source::core::CoreHandle;
use core::any::Any;
#[derive(Clone, Default)]
pub struct DataValueList {
    value: Vec<CoreHandle>,
}
impl DataValueList {
    pub const TYPE_KEY: DataType = DataType::List;
    pub const DEFAULT_VALUE: Option<&'static Vec<CoreHandle>> = None;
    pub fn value(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.value
    }
    pub fn items(&self) -> &Vec<CoreHandle> {
        &self.value
    }
    pub fn clear(&mut self) {
        self.value.clear()
    }
    pub fn add_item(&mut self, item: CoreHandle) {
        self.value.push(item)
    }
}
impl DataValue for DataValueList {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::List
    }
}
