use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Debug)]
pub struct DataValueViewModel {
    value: *mut (),
}
impl Default for DataValueViewModel {
    fn default() -> Self {
        Self {
            value: core::ptr::null_mut(),
        }
    }
}
impl DataValueViewModel {
    pub const TYPE_KEY: DataType = DataType::ViewModel;
    pub const DEFAULT_VALUE: *mut () = core::ptr::null_mut();
    pub fn value(&self) -> *mut () {
        self.value
    }
    pub fn set_value(&mut self, value: *mut ()) {
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
