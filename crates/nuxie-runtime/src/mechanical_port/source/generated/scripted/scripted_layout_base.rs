use crate::mechanical_port::source::scripted::{
    scripted_drawable::ScriptedDrawable, scripted_layout::ScriptedLayout,
};

pub struct ScriptedLayoutBase {
    pub base: ScriptedDrawable,
}

impl Default for ScriptedLayoutBase {
    fn default() -> Self {
        Self {
            base: ScriptedDrawable::default(),
        }
    }
}

impl ScriptedLayoutBase {
    pub const TYPE_KEY: u16 = 637;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 603 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ScriptedLayout {
        let mut cloned = ScriptedLayout::default();
        cloned.base.copy(self);
        cloned
    }
    pub fn copy(&mut self, object: &Self) {
        let mut base = std::mem::take(&mut self.base.base);
        base.copy(&object.base.base, &mut self.base);
        self.base.base = base;
    }
}

impl std::ops::Deref for ScriptedLayoutBase {
    type Target = ScriptedDrawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptedLayoutBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
