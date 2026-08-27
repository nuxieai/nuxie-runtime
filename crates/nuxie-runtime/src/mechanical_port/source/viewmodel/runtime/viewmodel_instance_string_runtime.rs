use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait StringValue: ViewModelInstanceValue {
    fn property_value(&self) -> &str;
    fn set_property_value(&self, value: String);
}
pub struct ViewModelInstanceStringRuntime<T: StringValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: StringValue> ViewModelInstanceStringRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn value(&self) -> &str {
        self.base.value().property_value()
    }
    pub fn set_value(&self, value: String) {
        self.base.value().set_property_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::String
    }
}
