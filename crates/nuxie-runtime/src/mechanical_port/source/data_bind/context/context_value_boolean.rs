use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::data_values::data_value_boolean::DataValueBoolean;
pub struct DataBindContextValueBoolean {
    base: DataBindContextValue,
}
impl DataBindContextValueBoolean {
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
            .downcast_ref::<DataValueBoolean>()
            .map_or(DataValueBoolean::DEFAULT_VALUE, DataValueBoolean::value);
        binding.set_bool(property_key, value)
    }
}
