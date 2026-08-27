use crate::mechanical_port::source::viewmodel::viewmodel_property_color::ViewModelPropertyColor;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub struct ViewModelPropertyColorBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyColorBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyColorBase {
    pub const TYPE_KEY: u16 = 440;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertyColor {
        let mut cloned = ViewModelPropertyColor::default();
        cloned.base.copy(self);
        cloned
    }
}
