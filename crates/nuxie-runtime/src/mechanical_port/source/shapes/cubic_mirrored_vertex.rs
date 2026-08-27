use crate::mechanical_port::source::{
    generated::shapes::cubic_mirrored_vertex_base::CubicMirroredVertexBase, math::vec2d::Vec2D,
    shapes::cubic_vertex::CubicVertexState,
};
pub struct CubicMirroredVertex {
    pub base: CubicMirroredVertexBase,
    pub cubic: CubicVertexState,
}
impl CubicMirroredVertex {
    fn point(&self) -> Vec2D {
        Vec2D::new(self.base.x(), self.base.y())
    }
    fn vector(&self) -> Vec2D {
        Vec2D::new(
            self.base.rotation().cos() * self.base.distance(),
            self.base.rotation().sin() * self.base.distance(),
        )
    }
    pub fn compute_in(&mut self) {
        self.cubic.in_point = self.point() - self.vector();
    }
    pub fn compute_out(&mut self) {
        self.cubic.out_point = self.point() + self.vector();
    }
    pub fn rotation_changed(&mut self) {
        self.cubic.in_valid = false;
        self.cubic.out_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn distance_changed(&mut self) {
        self.cubic.in_valid = false;
        self.cubic.out_valid = false;
        self.base.mark_geometry_dirty();
    }
}
