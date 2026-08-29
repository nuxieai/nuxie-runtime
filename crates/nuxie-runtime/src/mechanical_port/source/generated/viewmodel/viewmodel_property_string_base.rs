use crate::mechanical_port::source::viewmodel::viewmodel_property_string::ViewModelPropertyString;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property::ViewModelProperty,
};

pub struct ViewModelPropertyStringBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyStringBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyStringBase {
    pub const TYPE_KEY: u16 = 443;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyString {
        let mut cloned = ViewModelPropertyString::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyStringBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
