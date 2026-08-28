use crate::mechanical_port::source::viewmodel::viewmodel_property_asset::ViewModelPropertyAsset;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub struct ViewModelPropertyAssetBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyAssetBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyAssetBase {
    pub const TYPE_KEY: u16 = 584;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyAsset {
        let mut cloned = ViewModelPropertyAsset::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyAssetBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
