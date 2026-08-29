use crate::mechanical_port::source::{
    animation::focus_action::FocusAction, animation::focus_action_clear::FocusActionClear,
    core::binary_reader::BinaryReader,
};

pub struct FocusActionClearBase {
    pub base: FocusAction,
}

impl Default for FocusActionClearBase {
    fn default() -> Self {
        Self {
            base: FocusAction::default(),
        }
    }
}

impl FocusActionClearBase {
    pub const TYPE_KEY: u16 = 1037;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 671 | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FocusActionClear {
        let mut cloned = FocusActionClear::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for FocusActionClearBase {
    type Target = FocusAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FocusActionClearBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
