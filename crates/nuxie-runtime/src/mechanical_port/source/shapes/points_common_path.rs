use crate::mechanical_port::source::{
    generated::shapes::points_common_path_base::PointsCommonPathBase,
    shapes::shape_path_flags::ShapePathFlags,
};
pub struct PointsCommonPath {
    pub base: PointsCommonPathBase,
}
impl PointsCommonPath {
    pub fn is_path_closed(&self) -> bool {
        self.base.is_closed()
    }
    pub fn is_clockwise(&self) -> bool {
        self.base.path_flags() & ShapePathFlags::IsCounterClockwise as i32 == 0
    }
}
