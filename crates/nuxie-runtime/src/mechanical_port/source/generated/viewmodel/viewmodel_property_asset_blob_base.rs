use crate::mechanical_port::source::viewmodel::viewmodel_property_asset_blob::ViewModelPropertyAssetBlob;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property_asset::ViewModelPropertyAsset,
};

pub struct ViewModelPropertyAssetBlobBase {
    pub base: ViewModelPropertyAsset,
}

impl Default for ViewModelPropertyAssetBlobBase {
    fn default() -> Self {
        Self {
            base: ViewModelPropertyAsset::default(),
        }
    }
}

impl ViewModelPropertyAssetBlobBase {
    pub const TYPE_KEY: u16 = 1043;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 584 | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyAssetBlob {
        let mut cloned = ViewModelPropertyAssetBlob::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyAssetBlobBase {
    type Target = ViewModelPropertyAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyAssetBlobBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
