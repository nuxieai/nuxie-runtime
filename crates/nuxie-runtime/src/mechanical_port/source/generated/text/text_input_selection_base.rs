use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_input_drawable::TextInputDrawable,
    text::text_input_selection::TextInputSelection,
};

pub struct TextInputSelectionBase {
    pub base: TextInputDrawable,
}

impl Default for TextInputSelectionBase {
    fn default() -> Self {
        Self {
            base: TextInputDrawable::default(),
        }
    }
}

impl TextInputSelectionBase {
    pub const TYPE_KEY: u16 = 574;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 570 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TextInputSelection {
        let mut cloned = TextInputSelection::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for TextInputSelectionBase {
    type Target = TextInputDrawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextInputSelectionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
