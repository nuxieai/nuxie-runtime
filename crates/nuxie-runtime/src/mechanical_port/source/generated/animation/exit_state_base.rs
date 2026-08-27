use crate::mechanical_port::source::{
    animation::exit_state::ExitState, animation::layer_state::LayerState,
    core::binary_reader::BinaryReader,
};

pub struct ExitStateBase {
    pub base: LayerState,
}

impl Default for ExitStateBase {
    fn default() -> Self {
        Self {
            base: LayerState::default(),
        }
    }
}

impl ExitStateBase {
    pub const TYPE_KEY: u16 = 64;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ExitState {
        let mut cloned = ExitState::default();
        cloned.base.copy(self);
        cloned
    }
}
