use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::data_values::data_value_list::DataValueList;
pub struct DataBindContextValueList {
    base: DataBindContextValue,
}
super::context_value::impl_bind_context_value!(DataBindContextValueList);
impl DataBindContextValueList {
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
        if binding.has_target() {
            if let Some(value) = calculated.as_any().downcast_ref::<DataValueList>() {
                binding.update_list(value.items());
            }
        }
    }
}
