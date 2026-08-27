use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::points_common_path::PointsCommonPath,
    shapes::points_path::PointsPath,
};

pub struct PointsPathBase {
    pub base: PointsCommonPath,
}

impl Default for PointsPathBase {
    fn default() -> Self {
        Self {
            base: PointsCommonPath::default(),
        }
    }
}

impl PointsPathBase {
    pub const TYPE_KEY: u16 = 16;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 620 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> PointsPath {
        let mut cloned = PointsPath::default();
        cloned.base.copy(self);
        cloned
    }
}
