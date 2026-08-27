use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property_asset::ViewModelPropertyAsset,
};

pub struct ViewModelPropertyAssetFontBase {
    pub base: ViewModelPropertyAsset,
}

impl Default for ViewModelPropertyAssetFontBase {
    fn default() -> Self {
        Self {
            base: ViewModelPropertyAsset::default(),
        }
    }
}

impl ViewModelPropertyAssetFontBase {
    pub const TYPE_KEY: u16 = 1034;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0 | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
