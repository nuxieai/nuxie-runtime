use super::{data_type::DataType, data_value::DataValue};
use crate::mechanical_port::source::core::CoreHandle;
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueViewModel {
    value: Option<CoreHandle>,
}
impl DataValueViewModel {
    pub const TYPE_KEY: DataType = DataType::ViewModel;
    pub fn value(&self) -> Option<CoreHandle> {
        self.value.clone()
    }
    pub fn set_value(&mut self, value: Option<CoreHandle>) {
        self.value = value
    }
}
impl DataValue for DataValueViewModel {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::ViewModel
    }
}
