use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::axis::Axis, layout::axis_x::AxisX,
};

pub struct AxisXBase {
    pub base: Axis,
}

impl Default for AxisXBase {
    fn default() -> Self {
        Self {
            base: Axis::default(),
        }
    }
}

impl AxisXBase {
    pub const TYPE_KEY: u16 = 495;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 492 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> AxisX {
        let mut cloned = AxisX::default();
        cloned.base.copy(self);
        cloned
    }
}
