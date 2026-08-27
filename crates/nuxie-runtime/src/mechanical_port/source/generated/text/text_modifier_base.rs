use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub struct TextModifierBase {
    pub base: Component,
}

impl Default for TextModifierBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
        }
    }
}

impl TextModifierBase {
    pub const TYPE_KEY: u16 = 160;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
