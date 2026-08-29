use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::mechanical_port::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;

#[derive(Clone)]
pub struct ViewModelInstanceNumberRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceNumberRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Number).then_some(Self { base })
    }

    pub fn value(&self) -> f32 {
        self.base
            .handle()
            .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
            .unwrap_or_default()
    }

    pub fn set_value(&self, value: f32) {
        self.base
            .handle()
            .with_downcast_mut::<ViewModelInstanceNumber, _>(|property| property.set_value(value));
    }

    pub fn data_type(&self) -> DataType {
        DataType::Number
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
