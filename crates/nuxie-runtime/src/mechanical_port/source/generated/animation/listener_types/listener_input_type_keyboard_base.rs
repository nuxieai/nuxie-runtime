use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    animation::listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard,
    core::binary_reader::BinaryReader,
};

pub struct ListenerInputTypeKeyboardBase {
    pub base: ListenerInputType,
}

impl Default for ListenerInputTypeKeyboardBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
        }
    }
}

impl ListenerInputTypeKeyboardBase {
    pub const TYPE_KEY: u16 = 665;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ListenerInputTypeKeyboard {
        let mut cloned = ListenerInputTypeKeyboard::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ListenerInputTypeKeyboardBase {
    type Target = ListenerInputType;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeKeyboardBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
