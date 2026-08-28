use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::mechanical_port::source::viewmodel::viewmodel_instance_boolean::ViewModelInstanceBoolean;

#[derive(Clone)]
pub struct ViewModelInstanceBooleanRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceBooleanRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Boolean).then_some(Self { base })
    }

    pub fn value(&self) -> bool {
        self.base
            .handle()
            .with_downcast::<ViewModelInstanceBoolean, _>(ViewModelInstanceBoolean::value)
            .unwrap_or(false)
    }

    pub fn set_value(&self, value: bool) {
        self.base
            .handle()
            .with_downcast_mut::<ViewModelInstanceBoolean, _>(|property| property.set_value(value));
    }

    pub fn data_type(&self) -> DataType {
        DataType::Boolean
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
