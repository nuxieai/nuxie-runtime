use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::paint::linear_gradient::LinearGradient,
    shapes::paint::radial_gradient::RadialGradient,
};

pub struct RadialGradientBase {
    pub base: LinearGradient,
}

impl Default for RadialGradientBase {
    fn default() -> Self {
        Self {
            base: LinearGradient::default(),
        }
    }
}

impl RadialGradientBase {
    pub const TYPE_KEY: u16 = 17;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 22 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> RadialGradient {
        let mut cloned = RadialGradient::default();
        cloned.base.copy(self);
        cloned
    }
}
