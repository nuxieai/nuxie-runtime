use crate::mechanical_port::source::viewmodel::viewmodel_property_artboard::ViewModelPropertyArtboard;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property::ViewModelProperty,
};

pub struct ViewModelPropertyArtboardBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyArtboardBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyArtboardBase {
    pub const TYPE_KEY: u16 = 598;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyArtboard {
        let mut cloned = ViewModelPropertyArtboard::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyArtboardBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyArtboardBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
