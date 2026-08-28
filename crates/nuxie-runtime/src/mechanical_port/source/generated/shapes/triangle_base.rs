use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::parametric_path::ParametricPath,
    shapes::triangle::Triangle,
};

pub struct TriangleBase {
    pub base: ParametricPath,
}

impl Default for TriangleBase {
    fn default() -> Self {
        Self {
            base: ParametricPath::default(),
        }
    }
}

impl TriangleBase {
    pub const TYPE_KEY: u16 = 8;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 15 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> Triangle {
        let mut cloned = Triangle::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for TriangleBase {
    type Target = ParametricPath;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TriangleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
