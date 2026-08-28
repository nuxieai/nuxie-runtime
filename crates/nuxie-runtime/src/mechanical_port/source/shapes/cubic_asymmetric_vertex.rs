use crate::mechanical_port::source::{
    generated::shapes::cubic_asymmetric_vertex_base::CubicAsymmetricVertexBase,
    math::vec2d::Vec2D,
    shapes::{
        cubic_vertex::{CubicVertex, CubicVertexBehavior},
        vertex::{Vertex, VertexBehavior},
    },
};
#[derive(Default)]
pub struct CubicAsymmetricVertex {
    pub base: CubicAsymmetricVertexBase,
}

impl VertexBehavior for CubicAsymmetricVertex {
    fn vertex(&self) -> &Vertex {
        self.base.base.vertex()
    }
    fn vertex_mut(&mut self) -> &mut Vertex {
        self.base.base.vertex_mut()
    }
    fn mark_geometry_dirty(&mut self) {
        self.base.mark_geometry_dirty();
    }
}

impl CubicVertexBehavior for CubicAsymmetricVertex {
    fn cubic_vertex(&self) -> &CubicVertex {
        &self.base.base
    }
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        &mut self.base.base
    }
    fn compute_in(&mut self) {
        CubicAsymmetricVertex::compute_in(self);
    }
    fn compute_out(&mut self) {
        CubicAsymmetricVertex::compute_out(self);
    }
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
        self.base.base.state.in_point = self.point() - self.in_vector();
    }
    pub fn compute_out(&mut self) {
        self.base.base.state.out_point = self.point() + self.out_vector();
    }
    pub fn rotation_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn in_distance_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn out_distance_changed(&mut self) {
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
}
