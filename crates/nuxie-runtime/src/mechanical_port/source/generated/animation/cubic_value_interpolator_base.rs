use crate::mechanical_port::source::{
    animation::cubic_interpolator::CubicInterpolator,
    animation::cubic_value_interpolator::CubicValueInterpolator, core::binary_reader::BinaryReader,
};

pub struct CubicValueInterpolatorBase {
    pub base: CubicInterpolator,
}

impl Default for CubicValueInterpolatorBase {
    fn default() -> Self {
        Self {
            base: CubicInterpolator::default(),
        }
    }
}

impl CubicValueInterpolatorBase {
    pub const TYPE_KEY: u16 = 138;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 139 | 175)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> CubicValueInterpolator {
        let mut cloned = CubicValueInterpolator::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for CubicValueInterpolatorBase {
    type Target = CubicInterpolator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicValueInterpolatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
