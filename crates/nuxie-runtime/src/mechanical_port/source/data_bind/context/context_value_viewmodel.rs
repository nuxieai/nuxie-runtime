use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::{
    bindable_property_viewmodel::BindablePropertyViewModel,
    data_values::data_value_viewmodel::DataValueViewModel,
};
pub struct DataBindContextValueViewModel {
    base: DataBindContextValue,
}
impl DataBindContextValueViewModel {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn apply(
        &mut self,
        _property_key: u32,
        is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        self.base.sync_source_value(binding);
        let calculated = binding.convert(self.base.data_value().unwrap(), is_main_direction);
        let value = calculated
            .as_any()
            .downcast_ref::<DataValueViewModel>()
            .map_or(DataValueViewModel::DEFAULT_VALUE, DataValueViewModel::value);
        if binding.has_target() {
            binding.update_view_model(value);
            if binding.target_is_bindable_view_model() {
                binding.set_bindable_view_model(value);
                let key = binding.bindable_view_model_property_key();
                let pointer_key = if value.is_null() {
                    BindablePropertyViewModel::DEFAULT_VALUE
                } else {
                    binding.pointer_key(value)
                };
                binding.set_uint(key, pointer_key);
            }
        }
    }
}
