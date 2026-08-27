use crate::mechanical_port::source::animation::blend_state_1d_viewmodel::BlendState1DViewModel;

use crate::mechanical_port::source::{
    blend_state1_d::BlendState1D, core::binary_reader::BinaryReader,
};

pub struct BlendState1DViewModelBase {
    pub base: BlendState1D,
}

impl Default for BlendState1DViewModelBase {
    fn default() -> Self {
        Self {
            base: BlendState1D::default(),
        }
    }
}

impl BlendState1DViewModelBase {
    pub const TYPE_KEY: u16 = 528;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 527 | 72 | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> BlendState1DViewModel {
        let mut cloned = BlendState1DViewModel::default();
        cloned.base.copy(self);
        cloned
    }
}
