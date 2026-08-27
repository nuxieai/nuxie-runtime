use crate::mechanical_port::source::{
    animation::listener_action::ListenerAction, core::binary_reader::BinaryReader,
};

pub struct ListenerViewModelChangeBase {
    pub base: ListenerAction,
}

impl Default for ListenerViewModelChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
        }
    }
}

impl ListenerViewModelChangeBase {
    pub const TYPE_KEY: u16 = 487;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
