use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_asset::ViewModelInstanceAsset,
};

pub struct ViewModelInstanceAssetBlobBase {
    pub base: ViewModelInstanceAsset,
}

impl Default for ViewModelInstanceAssetBlobBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceAsset::default(),
        }
    }
}

impl ViewModelInstanceAssetBlobBase {
    pub const TYPE_KEY: u16 = 1044;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
