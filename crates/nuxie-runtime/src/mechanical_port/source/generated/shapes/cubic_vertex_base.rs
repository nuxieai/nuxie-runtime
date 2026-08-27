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
        matches!(type_key, Self::TYPE_KEY | 14 | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
