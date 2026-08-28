use crate::mechanical_port::source::viewmodel::viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob;

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
        matches!(type_key, Self::TYPE_KEY | 586 | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelInstanceAssetBlob {
        let mut cloned = ViewModelInstanceAssetBlob::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelInstanceAssetBlobBase {
    type Target = ViewModelInstanceAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceAssetBlobBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
