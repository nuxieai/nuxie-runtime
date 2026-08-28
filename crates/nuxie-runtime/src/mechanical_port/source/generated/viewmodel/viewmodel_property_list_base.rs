use crate::mechanical_port::source::viewmodel::viewmodel_property_list::ViewModelPropertyList;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub struct ViewModelPropertyListBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyListBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyListBase {
    pub const TYPE_KEY: u16 = 434;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyList {
        let mut cloned = ViewModelPropertyList::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ViewModelPropertyListBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyListBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
