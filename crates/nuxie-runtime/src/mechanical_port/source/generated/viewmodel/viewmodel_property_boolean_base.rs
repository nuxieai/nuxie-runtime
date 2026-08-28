use crate::mechanical_port::source::viewmodel::viewmodel_property_boolean::ViewModelPropertyBoolean;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property::ViewModelProperty,
};

pub struct ViewModelPropertyBooleanBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyBooleanBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyBooleanBase {
    pub const TYPE_KEY: u16 = 448;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyBoolean {
        let mut cloned = ViewModelPropertyBoolean::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyBooleanBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyBooleanBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
