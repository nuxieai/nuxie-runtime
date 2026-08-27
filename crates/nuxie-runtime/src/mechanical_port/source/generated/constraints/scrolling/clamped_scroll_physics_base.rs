use crate::mechanical_port::source::{
    constraints::scrolling::clamped_scroll_physics::ClampedScrollPhysics,
    constraints::scrolling::scroll_physics::ScrollPhysics, core::binary_reader::BinaryReader,
};

pub struct ClampedScrollPhysicsBase {
    pub base: ScrollPhysics,
}

impl Default for ClampedScrollPhysicsBase {
    fn default() -> Self {
        Self {
            base: ScrollPhysics::default(),
        }
    }
}

impl ClampedScrollPhysicsBase {
    pub const TYPE_KEY: u16 = 524;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 523 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ClampedScrollPhysics {
        let mut cloned = ClampedScrollPhysics::default();
        cloned.base.copy(self);
        cloned
    }
}
