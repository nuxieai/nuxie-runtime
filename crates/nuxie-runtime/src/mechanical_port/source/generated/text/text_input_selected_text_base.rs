use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_input_drawable::TextInputDrawable,
    text::text_input_selected_text::TextInputSelectedText,
};

pub struct TextInputSelectedTextBase {
    pub base: TextInputDrawable,
}

impl Default for TextInputSelectedTextBase {
    fn default() -> Self {
        Self {
            base: TextInputDrawable::default(),
        }
    }
}

impl TextInputSelectedTextBase {
    pub const TYPE_KEY: u16 = 575;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 570 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TextInputSelectedText {
        let mut cloned = TextInputSelectedText::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for TextInputSelectedTextBase {
    type Target = TextInputDrawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextInputSelectedTextBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
