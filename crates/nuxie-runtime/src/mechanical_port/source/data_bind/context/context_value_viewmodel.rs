use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::{
    bindable_property_viewmodel::BindablePropertyViewModel,
    data_values::data_value_viewmodel::DataValueViewModel,
};
pub struct DataBindContextValueViewModel {
    base: DataBindContextValue,
}
super::context_value::impl_bind_context_value!(DataBindContextValueViewModel);
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
            .and_then(DataValueViewModel::value);
        if binding.has_target() {
            binding.update_view_model(value.clone());
            if binding.target_is_bindable_view_model() {
                binding.set_bindable_view_model(value.clone());
                let key = binding.bindable_view_model_property_key();
                let pointer_key = value
                    .as_ref()
                    .map_or(BindablePropertyViewModel::DEFAULT_VALUE, |value| {
                        binding.pointer_key(value)
                    });
                binding.set_uint(key, pointer_key);
            }
        }
    }
}
