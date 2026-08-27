use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_asset::ViewModelInstanceAsset,
};

pub struct ViewModelInstanceAssetFontBase {
    pub base: ViewModelInstanceAsset,
}

impl Default for ViewModelInstanceAssetFontBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceAsset::default(),
        }
    }
}

impl ViewModelInstanceAssetFontBase {
    pub const TYPE_KEY: u16 = 1035;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
