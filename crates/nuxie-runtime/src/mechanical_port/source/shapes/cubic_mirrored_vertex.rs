use crate::mechanical_port::source::{
    generated::shapes::cubic_mirrored_vertex_base::CubicMirroredVertexBase,
    math::vec2d::Vec2D,
    shapes::{
        cubic_vertex::{CubicVertex, CubicVertexBehavior},
        vertex::{Vertex, VertexBehavior},
    },
};
impl std::ops::Deref for CubicMirroredVertex {
    type Target = CubicMirroredVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicMirroredVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CubicMirroredVertex {
    pub const TYPE_KEY: u16 = CubicMirroredVertexBase::TYPE_KEY;
}

#[derive(Default)]
pub struct CubicMirroredVertex {
    pub base: CubicMirroredVertexBase,
}

impl VertexBehavior for CubicMirroredVertex {
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

impl CubicVertexBehavior for CubicMirroredVertex {
    fn cubic_vertex(&self) -> &CubicVertex {
        &self.base.base
    }
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        &mut self.base.base
    }
    fn compute_in(&mut self) {
        CubicMirroredVertex::compute_in(self);
    }
    fn compute_out(&mut self) {
        CubicMirroredVertex::compute_out(self);
    }
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
        self.base.base.state.in_point = self.point() - self.vector();
    }
    pub fn compute_out(&mut self) {
        self.base.base.state.out_point = self.point() + self.vector();
    }
    pub fn rotation_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn distance_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
}
