use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::bindable_property_artboard::BindablePropertyArtboard,
    data_bind::bindable_property_id::BindablePropertyId,
};

pub struct BindablePropertyArtboardBase {
    pub base: BindablePropertyId,
}

impl Default for BindablePropertyArtboardBase {
    fn default() -> Self {
        Self {
            base: BindablePropertyId::default(),
        }
    }
}

impl BindablePropertyArtboardBase {
    pub const TYPE_KEY: u16 = 597;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 596 | 9)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> BindablePropertyArtboard {
        let mut cloned = BindablePropertyArtboard::default();
        cloned.base.copy(self);
        cloned
    }
}
