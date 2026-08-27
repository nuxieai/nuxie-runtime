use super::{
    context_target_value::{SourceKind, TargetKind},
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::data_value_artboard::DataValueArtboard;
pub struct DataBindContextValueArtboard {
    base: DataBindContextValue,
}
impl DataBindContextValueArtboard {
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
        if binding.source_kind() != SourceKind::Artboard {
            return;
        }
        if binding.target_kind() == TargetKind::ArtboardReferencer {
            binding.update_artboard(binding.source_artboard());
        } else {
            let calculated = binding.convert(self.base.data_value().unwrap(), is_main_direction);
            let value = calculated
                .as_any()
                .downcast_ref::<DataValueArtboard>()
                .map_or(DataValueArtboard::DEFAULT_VALUE, DataValueArtboard::value);
            binding.set_uint(property_key, value);
        }
    }
}
