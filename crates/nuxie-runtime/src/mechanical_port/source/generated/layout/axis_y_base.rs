use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::axis::Axis, layout::axis_y::AxisY,
};

pub struct AxisYBase {
    pub base: Axis,
}

impl Default for AxisYBase {
    fn default() -> Self {
        Self {
            base: Axis::default(),
        }
    }
}

impl AxisYBase {
    pub const TYPE_KEY: u16 = 494;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 492 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> AxisY {
        let mut cloned = AxisY::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for AxisYBase {
    type Target = Axis;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AxisYBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
