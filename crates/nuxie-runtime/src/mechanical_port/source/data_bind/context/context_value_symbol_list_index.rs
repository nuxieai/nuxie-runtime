use super::{
    context_target_value::FieldType,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::data_value_symbol_list_index::DataValueSymbolListIndex;
pub struct DataBindContextValueSymbolListIndex {
    base: DataBindContextValue,
}
super::context_value::impl_bind_context_value!(DataBindContextValueSymbolListIndex);
impl DataBindContextValueSymbolListIndex {
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
            .downcast_ref::<DataValueSymbolListIndex>()
            .map_or(0, DataValueSymbolListIndex::value);
        match binding.field_type() {
            FieldType::Double => binding.set_double(property_key, value as f32),
            FieldType::Uint => binding.set_uint(property_key, value),
            _ => {}
        }
    }
}
