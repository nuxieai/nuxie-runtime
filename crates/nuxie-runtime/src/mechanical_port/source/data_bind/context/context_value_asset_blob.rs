use super::{
    context_target_value::TargetKind,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
pub struct DataBindContextValueAssetBlob {
    base: DataBindContextValue,
}
impl DataBindContextValueAssetBlob {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn file_asset(
        &self,
        binding: &dyn ContextApplyBinding,
    ) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        binding.resolved_blob_asset()
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        _is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        match binding.target_kind() {
            TargetKind::BindableAsset => {
                binding.set_bindable_blob(binding.source_blob());
                binding.set_uint(property_key, binding.source_uint());
            }
            TargetKind::ViewModelAssetBlob => {
                let value = binding.source_uint();
                if value == u32::MAX {
                    binding.set_view_model_blob(binding.source_blob())
                } else {
                    binding.set_uint(property_key, value)
                }
            }
            _ => binding.set_uint(property_key, binding.source_uint()),
        }
    }
}
