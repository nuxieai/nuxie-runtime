use crate::mechanical_port::source::{
    animation::cubic_ease_interpolator::CubicEaseInterpolator,
    animation::cubic_interpolator::CubicInterpolator, core::binary_reader::BinaryReader,
};

pub struct CubicEaseInterpolatorBase {
    pub base: CubicInterpolator,
}

impl Default for CubicEaseInterpolatorBase {
    fn default() -> Self {
        Self {
            base: CubicInterpolator::default(),
        }
    }
}

impl CubicEaseInterpolatorBase {
    pub const TYPE_KEY: u16 = 28;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 139 | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> CubicEaseInterpolator {
        let mut cloned = CubicEaseInterpolator::default();
        cloned.base.copy(self);
        cloned
    }
}
