use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::data_values::data_value_color::DataValueColor;
pub struct DataBindContextValueColor {
    base: DataBindContextValue,
}
impl DataBindContextValueColor {
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
            .downcast_ref::<DataValueColor>()
            .map_or(DataValueColor::DEFAULT_VALUE, DataValueColor::value);
        binding.set_color(property_key, value)
    }
}
