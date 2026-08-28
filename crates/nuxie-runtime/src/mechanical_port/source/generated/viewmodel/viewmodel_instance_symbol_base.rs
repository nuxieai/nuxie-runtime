use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_instance_value::ViewModelInstanceValue,
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
        matches!(type_key, Self::TYPE_KEY | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for ViewModelInstanceSymbolBase {
    type Target = ViewModelInstanceValue;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceSymbolBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
