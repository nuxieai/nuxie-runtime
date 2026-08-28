use crate::mechanical_port::source::{
    animation::listener_input_change::ListenerInputChange,
    animation::listener_trigger_change::ListenerTriggerChange, core::binary_reader::BinaryReader,
};

pub struct ListenerTriggerChangeBase {
    pub base: ListenerInputChange,
}

impl Default for ListenerTriggerChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerInputChange::default(),
        }
    }
}

impl ListenerTriggerChangeBase {
    pub const TYPE_KEY: u16 = 115;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 116 | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ListenerTriggerChange {
        let mut cloned = ListenerTriggerChange::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ListenerTriggerChangeBase {
    type Target = ListenerInputChange;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerTriggerChangeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
