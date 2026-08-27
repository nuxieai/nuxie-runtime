use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait TriggerValue: ViewModelInstanceValue {
    fn trigger(&self);
}
pub struct ViewModelInstanceTriggerRuntime<T: TriggerValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: TriggerValue> ViewModelInstanceTriggerRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn trigger(&self) {
        self.base.value().trigger()
    }
    pub fn data_type(&self) -> DataType {
        DataType::Trigger
    }
}
