use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};

#[derive(Clone)]
pub struct ViewModelInstanceListIndexRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceListIndexRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::SymbolListIndex).then_some(Self { base })
    }
    pub fn value(&self) -> u32 {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_symbol_list_index()
                    .map(|property| property.base.property_value())
            })
            .flatten()
            .unwrap_or_default()
    }
    pub fn data_type(&self) -> DataType {
        DataType::SymbolListIndex
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
