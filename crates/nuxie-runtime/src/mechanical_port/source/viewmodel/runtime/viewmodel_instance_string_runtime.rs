use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::mechanical_port::source::viewmodel::viewmodel_instance_string::ViewModelInstanceString;

#[derive(Clone)]
pub struct ViewModelInstanceStringRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceStringRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::String).then_some(Self { base })
    }
    pub fn value(&self) -> String {
        self.base
            .handle()
            .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value)
            .unwrap_or_default()
    }
    pub fn set_value(&self, value: impl Into<String>) {
        let value = value.into();
        self.base
            .handle()
            .with_downcast_mut::<ViewModelInstanceString, _>(|property| property.set_value(value));
    }
    pub fn data_type(&self) -> DataType {
        DataType::String
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
