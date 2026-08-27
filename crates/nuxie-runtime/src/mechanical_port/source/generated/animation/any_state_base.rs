use crate::mechanical_port::source::{
    animation::any_state::AnyState, animation::layer_state::LayerState,
    core::binary_reader::BinaryReader,
};

pub struct AnyStateBase {
    pub base: LayerState,
}

impl Default for AnyStateBase {
    fn default() -> Self {
        Self {
            base: LayerState::default(),
        }
    }
}

impl AnyStateBase {
    pub const TYPE_KEY: u16 = 62;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> AnyState {
        let mut cloned = AnyState::default();
        cloned.base.copy(self);
        cloned
    }
}
