use crate::mechanical_port::source::{
    generated::shapes::points_common_path_base::PointsCommonPathBase,
    shapes::shape_path_flags::ShapePathFlags,
};
impl std::ops::Deref for PointsCommonPath {
    type Target = PointsCommonPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for PointsCommonPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl PointsCommonPath {
    pub const TYPE_KEY: u16 = PointsCommonPathBase::TYPE_KEY;
}

#[derive(Default)]
pub struct PointsCommonPath {
    pub base: PointsCommonPathBase,
}
impl PointsCommonPath {
    pub fn is_path_closed(&self) -> bool {
        self.base.is_closed()
    }
    pub fn is_clockwise(&self) -> bool {
        self.base.path_flags() & ShapePathFlags::IsCounterClockwise as u32 == 0
    }
}
