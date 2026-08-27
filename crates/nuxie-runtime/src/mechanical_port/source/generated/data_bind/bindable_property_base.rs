use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub struct BindablePropertyBase {
    pub base: Core,
}

impl Default for BindablePropertyBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
        }
    }
}

impl BindablePropertyBase {
    pub const TYPE_KEY: u16 = 9;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {}
    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        false
    }
}
