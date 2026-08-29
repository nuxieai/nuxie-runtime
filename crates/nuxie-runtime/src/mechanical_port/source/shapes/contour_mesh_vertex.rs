use crate::mechanical_port::source::{
    generated::shapes::contour_mesh_vertex_base::ContourMeshVertexBase,
    shapes::vertex::{Vertex, VertexBehavior},
};

impl std::ops::Deref for ContourMeshVertex {
    type Target = ContourMeshVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ContourMeshVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ContourMeshVertex {
    pub const TYPE_KEY: u16 = ContourMeshVertexBase::TYPE_KEY;
}

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
