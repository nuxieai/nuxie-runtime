use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait BooleanValue: ViewModelInstanceValue {
    fn property_value(&self) -> bool;
    fn set_property_value(&self, value: bool);
}
pub struct ViewModelInstanceBooleanRuntime<T: BooleanValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: BooleanValue> ViewModelInstanceBooleanRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn value(&self) -> bool {
        self.base.value().property_value()
    }
    pub fn set_value(&self, value: bool) {
        self.base.value().set_property_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::Boolean
    }
}
