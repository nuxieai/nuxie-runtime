use crate::mechanical_port::source::{core::binary_reader::BinaryReader, drawable::Drawable};

pub struct TextInputDrawableBase {
    pub base: Drawable,
}

impl Default for TextInputDrawableBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
        }
    }
}

impl TextInputDrawableBase {
    pub const TYPE_KEY: u16 = 570;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for TextInputDrawableBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextInputDrawableBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
