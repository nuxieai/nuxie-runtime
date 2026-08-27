use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_style::TextStyle,
    text::text_style_paint::TextStylePaint,
};

pub struct TextStylePaintBase {
    pub base: TextStyle,
}

impl Default for TextStylePaintBase {
    fn default() -> Self {
        Self {
            base: TextStyle::default(),
        }
    }
}

impl TextStylePaintBase {
    pub const TYPE_KEY: u16 = 137;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 573 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TextStylePaint {
        let mut cloned = TextStylePaint::default();
        cloned.base.copy(self);
        cloned
    }
}
