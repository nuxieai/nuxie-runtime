use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub struct ViewModelPropertyEnumBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertyEnumBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertyEnumBase {
    pub const TYPE_KEY: u16 = 509;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
