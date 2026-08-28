use crate::mechanical_port::source::viewmodel::viewmodel_property_number::ViewModelPropertyNumber;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property::ViewModelProperty,
};

pub struct ViewModelPropertyNumberBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyNumberBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyNumberBase {
    pub const TYPE_KEY: u16 = 431;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyNumber {
        let mut cloned = ViewModelPropertyNumber::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyNumberBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyNumberBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
