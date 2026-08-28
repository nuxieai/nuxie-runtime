use crate::mechanical_port::source::{core::binary_reader::BinaryReader, shapes::vertex::Vertex};

pub struct PathVertexBase {
    pub base: Vertex,
}

impl Default for PathVertexBase {
    fn default() -> Self {
        Self {
            base: Vertex::default(),
        }
    }
}

impl PathVertexBase {
    pub const TYPE_KEY: u16 = 14;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for PathVertexBase {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for PathVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
