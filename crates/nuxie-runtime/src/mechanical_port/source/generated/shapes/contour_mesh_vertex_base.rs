use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::contour_mesh_vertex::ContourMeshVertex,
    shapes::mesh_vertex::MeshVertex,
};

pub struct ContourMeshVertexBase {
    pub base: MeshVertex,
}

impl Default for ContourMeshVertexBase {
    fn default() -> Self {
        Self {
            base: MeshVertex::default(),
        }
    }
}

impl ContourMeshVertexBase {
    pub const TYPE_KEY: u16 = 111;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 108 | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ContourMeshVertex {
        let mut cloned = ContourMeshVertex::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ContourMeshVertexBase {
    type Target = MeshVertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ContourMeshVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
