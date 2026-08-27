use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait ListIndexValue: ViewModelInstanceValue {
    fn property_value(&self) -> u32;
}
pub struct ViewModelInstanceListIndexRuntime<T: ListIndexValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: ListIndexValue> ViewModelInstanceListIndexRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn value(&self) -> u32 {
        self.base.value().property_value()
    }
    pub fn data_type(&self) -> DataType {
        DataType::SymbolListIndex
    }
}
