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
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for TextShapeModifierBase {
    type Target = TextModifier;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextShapeModifierBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
