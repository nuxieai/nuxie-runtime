use super::{
    context_target_value::TargetKind,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
pub struct DataBindContextValueAssetFont {
    base: DataBindContextValue,
}
impl DataBindContextValueAssetFont {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn file_asset(&self, binding: &dyn ContextApplyBinding) -> *mut () {
        binding.resolved_font_asset()
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        _is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        match binding.target_kind() {
            TargetKind::TextStyle => {
                let resolved = self.file_asset(binding);
                if !resolved.is_null() {
                    binding.set_target_font_asset(resolved)
                } else if !binding.source_font_asset().is_null() && !binding.source_font().is_null()
                {
                    binding.set_target_font_asset(binding.source_font_asset())
                }
            }
            TargetKind::BindableAsset => {
                binding.set_bindable_font(binding.source_font());
                binding.set_uint(property_key, binding.source_uint());
            }
            TargetKind::ViewModelAssetFont => {
                let value = binding.source_uint();
                if value == u32::MAX {
                    binding.set_view_model_font(binding.source_font())
                } else {
                    binding.set_uint(property_key, value)
                }
            }
            _ => binding.set_uint(property_key, binding.source_uint()),
        }
    }
}
