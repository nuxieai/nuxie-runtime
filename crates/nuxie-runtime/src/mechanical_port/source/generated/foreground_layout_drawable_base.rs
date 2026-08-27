use crate::mechanical_port::source::{
    drawable::Drawable, foreground_layout_drawable::ForegroundLayoutDrawable,
};

pub struct ForegroundLayoutDrawableBase {
    pub base: Drawable,
}
impl Default for ForegroundLayoutDrawableBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
        }
    }
}
impl ForegroundLayoutDrawableBase {
    pub const TYPE_KEY: u16 = 513;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> ForegroundLayoutDrawable {
        let mut cloned = ForegroundLayoutDrawable::default();
        cloned.base.copy(self);
        cloned
    }
}
