use crate::mechanical_port::source::{
    animation::blend_state::BlendState, core::binary_reader::BinaryReader,
};

pub struct BlendState1DBase {
    pub base: BlendState,
}

impl Default for BlendState1DBase {
    fn default() -> Self {
        Self {
            base: BlendState::default(),
        }
    }
}

impl BlendState1DBase {
    pub const TYPE_KEY: u16 = 527;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 72 | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for BlendState1DBase {
    type Target = BlendState;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendState1DBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
