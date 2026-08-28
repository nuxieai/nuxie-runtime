use super::{
    context_target_value::FieldType,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::data_value_enum::DataValueEnum;
pub struct DataBindContextValueEnum {
    base: DataBindContextValue,
}
impl DataBindContextValueEnum {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        self.base.sync_source_value(binding);
        let calculated = binding.convert(self.base.data_value().unwrap(), is_main_direction);
        let Some(value) = calculated.as_any().downcast_ref::<DataValueEnum>() else {
            return;
        };
        if binding.field_type() == FieldType::Uint && binding.target_is_solo() {
            if let Some(data_enum) = value.data_enum() {
                binding.solo_update_by_name(data_enum.value(value.value()));
            }
        } else {
            binding.set_uint(property_key, value.value());
        }
    }
}
