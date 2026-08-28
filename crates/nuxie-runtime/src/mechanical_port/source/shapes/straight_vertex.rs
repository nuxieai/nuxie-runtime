use crate::mechanical_port::source::{
    generated::shapes::straight_vertex_base::StraightVertexBase,
    shapes::vertex::{Vertex, VertexBehavior},
};
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
}
