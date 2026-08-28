use crate::mechanical_port::source::{
    animation::blend_state::BlendState, animation::blend_state_direct::BlendStateDirect,
    core::binary_reader::BinaryReader,
};

pub struct BlendStateDirectBase {
    pub base: BlendState,
}

impl Default for BlendStateDirectBase {
    fn default() -> Self {
        Self {
            base: BlendState::default(),
        }
    }
}

impl BlendStateDirectBase {
    pub const TYPE_KEY: u16 = 73;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 72 | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> BlendStateDirect {
        let mut cloned = BlendStateDirect::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for BlendStateDirectBase {
    type Target = BlendState;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendStateDirectBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
