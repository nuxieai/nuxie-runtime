use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::ellipse::Ellipse,
    shapes::parametric_path::ParametricPath,
};

pub struct EllipseBase {
    pub base: ParametricPath,
}

impl Default for EllipseBase {
    fn default() -> Self {
        Self {
            base: ParametricPath::default(),
        }
    }
}

impl EllipseBase {
    pub const TYPE_KEY: u16 = 4;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 15 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> Ellipse {
        let mut cloned = Ellipse::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for EllipseBase {
    type Target = ParametricPath;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for EllipseBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
