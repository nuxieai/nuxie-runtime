use crate::mechanical_port::source::{
    generated::shapes::straight_vertex_base::StraightVertexBase,
    shapes::vertex::{Vertex, VertexBehavior},
};
impl std::ops::Deref for StraightVertex {
    type Target = StraightVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StraightVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl StraightVertex {
    pub const TYPE_KEY: u16 = StraightVertexBase::TYPE_KEY;
}

#[derive(Default)]
pub struct StraightVertex {
    pub base: StraightVertexBase,
}
impl VertexBehavior for StraightVertex {
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
impl StraightVertex {
    pub fn radius_changed(&mut self) {
        self.base.mark_geometry_dirty();
    }
    pub fn set_x(&mut self, value: f32) {
        if self.vertex_mut().base.set_x_value(value) {
            VertexBehavior::x_changed(self);
            self.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::X_PROPERTY_KEY);
        }
    }
    pub fn set_y(&mut self, value: f32) {
        if self.vertex_mut().base.set_y_value(value) {
            VertexBehavior::y_changed(self);
            self.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::Y_PROPERTY_KEY);
        }
    }
    pub fn set_radius(&mut self, value: f32) {
        if self.base.set_radius_value(value) {
            self.radius_changed();
            self.vertex_mut()
                .notify_property_changed(StraightVertexBase::RADIUS_PROPERTY_KEY);
        }
    }
}
