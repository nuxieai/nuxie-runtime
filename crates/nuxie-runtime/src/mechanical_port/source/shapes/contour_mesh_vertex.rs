use crate::mechanical_port::source::{
    generated::shapes::contour_mesh_vertex_base::ContourMeshVertexBase,
    shapes::vertex::{Vertex, VertexBehavior},
};

#[derive(Default)]
pub struct ContourMeshVertex {
    pub base: ContourMeshVertexBase,
}
impl VertexBehavior for ContourMeshVertex {
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
