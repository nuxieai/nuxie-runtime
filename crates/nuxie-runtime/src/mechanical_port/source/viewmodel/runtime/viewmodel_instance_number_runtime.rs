use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait NumberValue: ViewModelInstanceValue {
    fn property_value(&self) -> f32;
    fn set_property_value(&self, value: f32);
}
pub struct ViewModelInstanceNumberRuntime<T: NumberValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: NumberValue> ViewModelInstanceNumberRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn value(&self) -> f32 {
        self.base.value().property_value()
    }
    pub fn set_value(&self, value: f32) {
        self.base.value().set_property_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::Number
    }
}
