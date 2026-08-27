use crate::mechanical_port::source::{
    animation::focus_action::FocusAction, animation::listener_action::ListenerAction,
    core::binary_reader::BinaryReader,
};

pub struct FocusActionBase {
    pub base: ListenerAction,
}

impl Default for FocusActionBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
        }
    }
}

impl FocusActionBase {
    pub const TYPE_KEY: u16 = 671;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FocusAction {
        let mut cloned = FocusAction::default();
        cloned.base.copy(self);
        cloned
    }
}
