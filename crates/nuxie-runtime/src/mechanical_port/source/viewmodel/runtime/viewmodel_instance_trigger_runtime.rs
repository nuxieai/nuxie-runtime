use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};

#[derive(Clone)]
pub struct ViewModelInstanceTriggerRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceTriggerRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Trigger).then_some(Self { base })
    }

    pub fn trigger(&self) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_trigger_mut() {
                property.trigger();
            }
        });
    }

    pub fn data_type(&self) -> DataType {
        DataType::Trigger
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
