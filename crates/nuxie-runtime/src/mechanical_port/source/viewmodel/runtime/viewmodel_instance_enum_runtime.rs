use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};

#[derive(Clone)]
pub struct ViewModelInstanceEnumRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceEnumRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Enum).then_some(Self { base })
    }

    pub fn value(&self) -> String {
        self.base
            .handle()
            .with(|value| {
                value
                    .as_view_model_instance_enum()
                    .map(|value| value.value())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn set_value(&self, value: impl AsRef<str>) -> bool {
        self.base
            .handle()
            .with_mut(|property| {
                property
                    .as_view_model_instance_enum_mut()
                    .is_some_and(|property| property.set_value_named(value.as_ref()))
            })
            .unwrap_or(false)
    }

    pub fn value_index(&self) -> u32 {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_enum()
                    .map(|property| property.base.property_value())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn set_value_index(&self, index: u32) -> bool {
        self.base
            .handle()
            .with_mut(|property| {
                property
                    .as_view_model_instance_enum_mut()
                    .is_some_and(|property| property.set_value_at(index))
            })
            .unwrap_or(false)
    }

    pub fn values(&self) -> Vec<String> {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_enum()
                    .map(|property| property.values())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn enum_type(&self) -> String {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_enum()
                    .map(|property| property.enum_type())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn data_type(&self) -> DataType {
        DataType::Enum
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
