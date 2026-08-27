use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    animation::listener_types::listener_input_type_text::ListenerInputTypeText,
    core::binary_reader::BinaryReader,
};

pub struct ListenerInputTypeTextBase {
    pub base: ListenerInputType,
}

impl Default for ListenerInputTypeTextBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
        }
    }
}

impl ListenerInputTypeTextBase {
    pub const TYPE_KEY: u16 = 666;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ListenerInputTypeText {
        let mut cloned = ListenerInputTypeText::default();
        cloned.base.copy(self);
        cloned
    }
}
