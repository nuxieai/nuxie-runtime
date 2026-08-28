use crate::mechanical_port::source::{
    constraints::scale_constraint::ScaleConstraint,
    constraints::transform_component_constraint_y::TransformComponentConstraintY,
    core::binary_reader::BinaryReader,
};

pub struct ScaleConstraintBase {
    pub base: TransformComponentConstraintY,
}

impl Default for ScaleConstraintBase {
    fn default() -> Self {
        Self {
            base: TransformComponentConstraintY::default(),
        }
    }
}

impl ScaleConstraintBase {
    pub const TYPE_KEY: u16 = 88;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 86 | 85 | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ScaleConstraint {
        let mut cloned = ScaleConstraint::default();
        let mut callbacks = ScaleConstraint::default();
        cloned.base.copy(self, &mut callbacks);
        cloned
    }
}

impl std::ops::Deref for ScaleConstraintBase {
    type Target = TransformComponentConstraintY;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScaleConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
