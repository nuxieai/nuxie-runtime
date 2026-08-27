use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait EnumValue: ViewModelInstanceValue {
    fn property_value(&self) -> u32;
    fn set_value(&self, value: String);
    fn data_values(&self) -> Vec<String>;
    fn enum_name(&self) -> Option<&str>;
}
pub struct ViewModelInstanceEnumRuntime<T: EnumValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: EnumValue> ViewModelInstanceEnumRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    fn data_values(&self) -> Vec<String> {
        self.base.value().data_values()
    }
    pub fn value(&self) -> String {
        let values = self.data_values();
        values
            .get(self.base.value().property_value() as usize)
            .cloned()
            .unwrap_or_default()
    }
    pub fn set_value(&self, value: String) {
        self.base.value().set_value(value)
    }
    pub fn value_index(&self) -> u32 {
        let index = self.base.value().property_value();
        if (index as usize) < self.data_values().len() {
            index
        } else {
            0
        }
    }
    pub fn set_value_index(&self, index: u32) {
        if let Some(value) = self.data_values().get(index as usize) {
            self.base.value().set_value(value.clone())
        }
    }
    pub fn values(&self) -> Vec<String> {
        self.data_values()
    }
    pub fn enum_type(&self) -> String {
        let name = self.base.value().enum_name();
        assert!(name.is_some());
        let name = name.unwrap();
        name.to_owned()
    }
    pub fn data_type(&self) -> DataType {
        DataType::Enum
    }
}
