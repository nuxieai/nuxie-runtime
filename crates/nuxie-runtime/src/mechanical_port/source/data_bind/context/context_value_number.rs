use super::{
    context_target_value::FieldType,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::data_value_number::DataValueNumber;
pub struct DataBindContextValueNumber {
    base: DataBindContextValue,
}
super::context_value::impl_bind_context_value!(DataBindContextValueNumber);
impl DataBindContextValueNumber {
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
            .downcast_ref::<DataValueNumber>()
            .map_or(DataValueNumber::DEFAULT_VALUE, DataValueNumber::value);
        match binding.field_type() {
            FieldType::Double => binding.set_double(property_key, value),
            FieldType::Uint => {
                if binding.target_is_solo() {
                    binding.solo_update_by_index(value.round() as usize)
                } else {
                    binding.set_uint(
                        property_key,
                        if value < 0.0 { 0 } else { value.round() as u32 },
                    )
                }
            }
            _ => {}
        }
    }
}
