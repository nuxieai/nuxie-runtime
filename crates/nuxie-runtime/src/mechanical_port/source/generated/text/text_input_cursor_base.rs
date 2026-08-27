use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_input_cursor::TextInputCursor,
    text::text_input_drawable::TextInputDrawable,
};

pub struct TextInputCursorBase {
    pub base: TextInputDrawable,
}

impl Default for TextInputCursorBase {
    fn default() -> Self {
        Self {
            base: TextInputDrawable::default(),
        }
    }
}

impl TextInputCursorBase {
    pub const TYPE_KEY: u16 = 571;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 570 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TextInputCursor {
        let mut cloned = TextInputCursor::default();
        cloned.base.copy(self);
        cloned
    }
}
