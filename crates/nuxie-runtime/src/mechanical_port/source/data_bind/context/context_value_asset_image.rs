use super::{
    context_target_value::TargetKind,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
pub struct DataBindContextValueAssetImage {
    base: DataBindContextValue,
}
impl DataBindContextValueAssetImage {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn file_asset(&self, binding: &dyn ContextApplyBinding) -> *mut () {
        binding.resolved_image_asset()
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        _is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        match binding.target_kind() {
            TargetKind::Image => {
                let resolved = self.file_asset(binding);
                let asset = if resolved.is_null() {
                    binding.source_image_asset()
                } else {
                    resolved
                };
                binding.set_target_image_asset(asset);
            }
            TargetKind::BindableAsset => {
                binding.set_bindable_image(binding.source_image());
                binding.set_uint(property_key, binding.source_uint());
            }
            TargetKind::ViewModelAssetImage => {
                let value = binding.source_uint();
                if value == u32::MAX {
                    binding.set_view_model_image(binding.source_image())
                } else {
                    binding.set_uint(property_key, value)
                }
            }
            _ => binding.set_uint(property_key, binding.source_uint()),
        }
    }
}
