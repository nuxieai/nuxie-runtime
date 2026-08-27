use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_value::ViewModelInstanceValue,
};

pub struct ViewModelInstanceSymbolBase {
    pub base: ViewModelInstanceValue,
}

impl Default for ViewModelInstanceSymbolBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValue::default(),
        }
    }
}

impl ViewModelInstanceSymbolBase {
    pub const TYPE_KEY: u16 = 565;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
