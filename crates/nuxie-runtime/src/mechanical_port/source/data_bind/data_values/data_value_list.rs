use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
use std::rc::Rc;
pub trait ViewModelInstanceListItem: Any {}
#[derive(Default)]
pub struct DataValueList {
    value: Vec<Rc<dyn ViewModelInstanceListItem>>,
}
impl DataValueList {
    pub const TYPE_KEY: DataType = DataType::List;
    pub const DEFAULT_VALUE: Option<&'static Vec<Rc<dyn ViewModelInstanceListItem>>> = None;
    pub fn value(&mut self) -> &mut Vec<Rc<dyn ViewModelInstanceListItem>> {
        &mut self.value
    }
    pub fn items(&self) -> &Vec<Rc<dyn ViewModelInstanceListItem>> {
        &self.value
    }
    pub fn clear(&mut self) {
        self.value.clear()
    }
    pub fn add_item(&mut self, item: Rc<dyn ViewModelInstanceListItem>) {
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
