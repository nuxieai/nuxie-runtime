use crate::mechanical_port::source::{
    generated::shapes::cubic_detached_vertex_base::CubicDetachedVertexBase,
    math::vec2d::Vec2D,
    shapes::{
        cubic_vertex::{CubicVertex, CubicVertexBehavior},
        vertex::{Vertex, VertexBehavior},
    },
};
impl std::ops::Deref for CubicDetachedVertex {
    type Target = CubicDetachedVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicDetachedVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CubicDetachedVertex {
    pub const TYPE_KEY: u16 = CubicDetachedVertexBase::TYPE_KEY;
}

#[derive(Default)]
pub struct CubicDetachedVertex {
    pub base: CubicDetachedVertexBase,
}

impl VertexBehavior for CubicDetachedVertex {
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

impl CubicVertexBehavior for CubicDetachedVertex {
    fn cubic_vertex(&self) -> &CubicVertex {
        &self.base.base
    }
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        &mut self.base.base
    }
    fn compute_in(&mut self) {
        CubicDetachedVertex::compute_in(self);
    }
    fn compute_out(&mut self) {
        CubicDetachedVertex::compute_out(self);
    }
}
impl CubicDetachedVertex {
    fn point(&self) -> Vec2D {
        Vec2D::new(self.base.x(), self.base.y())
    }
    pub fn set_x(&mut self, value: f32) {
        if self.vertex_mut().base.set_x_value(value) {
            CubicVertexBehavior::x_changed(self);
            self.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::X_PROPERTY_KEY);
        }
    }
    pub fn set_y(&mut self, value: f32) {
        if self.vertex_mut().base.set_y_value(value) {
            CubicVertexBehavior::y_changed(self);
            self.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::Y_PROPERTY_KEY);
        }
    }
    fn in_vector(&self) -> Vec2D {
        Vec2D::new(
            self.base.in_rotation().cos() * self.base.in_distance(),
            self.base.in_rotation().sin() * self.base.in_distance(),
        )
    }
    fn out_vector(&self) -> Vec2D {
        Vec2D::new(
            self.base.out_rotation().cos() * self.base.out_distance(),
            self.base.out_rotation().sin() * self.base.out_distance(),
        )
    }
    pub fn compute_in(&mut self) {
        self.base.base.state.in_point = self.point() + self.in_vector();
    }
    pub fn compute_out(&mut self) {
        self.base.base.state.out_point = self.point() + self.out_vector();
    }
    pub fn in_rotation_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn in_distance_changed(&mut self) {
        self.base.base.state.in_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn out_rotation_changed(&mut self) {
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
    pub fn out_distance_changed(&mut self) {
        self.base.base.state.out_valid = false;
        self.base.mark_geometry_dirty();
    }
}
