use crate::mechanical_port::source::{
    generated::shapes::cubic_asymmetric_vertex_base::CubicAsymmetricVertexBase, math::vec2d::Vec2D,
    shapes::cubic_vertex::CubicVertexState,
};
pub struct CubicAsymmetricVertex {
    pub base: CubicAsymmetricVertexBase,
    pub cubic: CubicVertexState,
}
impl CubicAsymmetricVertex {
    fn point(&self) -> Vec2D {
        Vec2D::new(self.base.x(), self.base.y())
    }
    fn in_vector(&self) -> Vec2D {
        Vec2D::new(
            self.base.rotation().cos() * self.base.in_distance(),
            self.base.rotation().sin() * self.base.in_distance(),
        )
    }
    fn out_vector(&self) -> Vec2D {
        Vec2D::new(
            self.base.rotation().cos() * self.base.out_distance(),
            self.base.rotation().sin() * self.base.out_distance(),
        )
    }
    pub fn compute_in(&mut self) {
        self.cubic.in_point = self.point() - self.in_vector();
    }
    pub fn compute_out(&mut self) {
        self.cubic.out_point = self.point() + self.out_vector();
    }
    pub fn rotation_changed(&mut self) {
        self.cubic.in_valid = false;
        self.cubic.out_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn in_distance_changed(&mut self) {
        self.cubic.in_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn out_distance_changed(&mut self) {
        self.cubic.out_valid = false;
        self.base.mark_geometry_dirty();
    }
}
