use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::path_vertex::PathVertex,
};

pub struct CubicVertexBase {
    pub base: PathVertex,
}

impl Default for CubicVertexBase {
    fn default() -> Self {
        Self {
            base: PathVertex::default(),
        }
    }
}

impl CubicVertexBase {
    pub const TYPE_KEY: u16 = 36;

    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for CubicVertexBase {
    type Target = PathVertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
