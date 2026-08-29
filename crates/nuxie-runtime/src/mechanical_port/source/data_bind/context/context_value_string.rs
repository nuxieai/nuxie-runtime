use super::{
    context_target_value::FieldType,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::data_value_string::DataValueString;
pub struct DataBindContextValueString {
    base: DataBindContextValue,
}
super::context_value::impl_bind_context_value!(DataBindContextValueString);
impl DataBindContextValueString {
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
        let value = calculated
            .as_any()
            .downcast_ref::<DataValueString>()
            .map_or_else(String::new, |value| value.value().to_owned());
        if binding.field_type() == FieldType::Uint {
            if binding.target_is_solo() {
                binding.solo_update_by_name(value);
            }
        } else {
            binding.set_string(property_key, value);
        }
    }
}
