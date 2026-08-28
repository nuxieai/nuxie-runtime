use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    animation::listener_types::listener_input_type_gamepad::ListenerInputTypeGamepad,
    core::binary_reader::BinaryReader,
};

pub struct ListenerInputTypeGamepadBase {
    pub base: ListenerInputType,
}

impl Default for ListenerInputTypeGamepadBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
        }
    }
}

impl ListenerInputTypeGamepadBase {
    pub const TYPE_KEY: u16 = 973;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ListenerInputTypeGamepad {
        let mut cloned = ListenerInputTypeGamepad::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ListenerInputTypeGamepadBase {
    type Target = ListenerInputType;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeGamepadBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
