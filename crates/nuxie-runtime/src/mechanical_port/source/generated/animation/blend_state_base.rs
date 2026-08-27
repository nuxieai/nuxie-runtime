use crate::mechanical_port::source::{
    animation::layer_state::LayerState, core::binary_reader::BinaryReader,
};

pub struct BlendStateBase {
    pub base: LayerState,
}

impl Default for BlendStateBase {
    fn default() -> Self {
        Self {
            base: LayerState::default(),
        }
    }
}

impl BlendStateBase {
    pub const TYPE_KEY: u16 = 72;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
