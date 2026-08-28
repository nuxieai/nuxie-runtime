use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub struct ViewModelPropertySymbolBase {
    pub base: ViewModelProperty,
}

impl Default for ViewModelPropertySymbolBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
        }
    }
}

impl ViewModelPropertySymbolBase {
    pub const TYPE_KEY: u16 = 563;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for ViewModelPropertySymbolBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertySymbolBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
