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
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for BlendStateBase {
    type Target = LayerState;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendStateBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
