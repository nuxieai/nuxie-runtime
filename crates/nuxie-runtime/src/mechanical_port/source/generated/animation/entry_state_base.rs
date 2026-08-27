use crate::mechanical_port::source::{
    animation::entry_state::EntryState, animation::layer_state::LayerState,
    core::binary_reader::BinaryReader,
};

pub struct EntryStateBase {
    pub base: LayerState,
}

impl Default for EntryStateBase {
    fn default() -> Self {
        Self {
            base: LayerState::default(),
        }
    }
}

impl EntryStateBase {
    pub const TYPE_KEY: u16 = 63;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> EntryState {
        let mut cloned = EntryState::default();
        cloned.base.copy(self);
        cloned
    }
}
