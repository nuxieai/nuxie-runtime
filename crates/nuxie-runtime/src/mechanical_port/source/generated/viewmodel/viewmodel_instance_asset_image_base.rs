use crate::mechanical_port::source::viewmodel::viewmodel_instance_asset_image::ViewModelInstanceAssetImage;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_instance_asset::ViewModelInstanceAsset,
};

pub struct ViewModelInstanceAssetImageBase {
    pub base: ViewModelInstanceAsset,
}

impl Default for ViewModelInstanceAssetImageBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceAsset::default(),
        }
    }
}

impl ViewModelInstanceAssetImageBase {
    pub const TYPE_KEY: u16 = 587;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 586 | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelInstanceAssetImage {
        let mut cloned = ViewModelInstanceAssetImage::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ViewModelInstanceAssetImageBase {
    type Target = ViewModelInstanceAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceAssetImageBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
