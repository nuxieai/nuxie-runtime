use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_modifier::TextModifier,
};

pub struct TextShapeModifierBase {
    pub base: TextModifier,
}

impl Default for TextShapeModifierBase {
    fn default() -> Self {
        Self {
            base: TextModifier::default(),
        }
    }
}

impl TextShapeModifierBase {
    pub const TYPE_KEY: u16 = 161;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 160 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
