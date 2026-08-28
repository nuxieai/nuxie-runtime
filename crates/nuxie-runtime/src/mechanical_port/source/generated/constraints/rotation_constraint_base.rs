use crate::mechanical_port::source::{
    constraints::rotation_constraint::RotationConstraint,
    constraints::transform_component_constraint::TransformComponentConstraint,
    core::binary_reader::BinaryReader,
};

pub struct RotationConstraintBase {
    pub base: TransformComponentConstraint,
}

impl Default for RotationConstraintBase {
    fn default() -> Self {
        Self {
            base: TransformComponentConstraint::default(),
        }
    }
}

impl RotationConstraintBase {
    pub const TYPE_KEY: u16 = 89;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 85 | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> RotationConstraint {
        let mut cloned = RotationConstraint::default();
        let mut callbacks = RotationConstraint::default();
        cloned.base.copy(self, &mut callbacks);
        cloned
    }
}

impl std::ops::Deref for RotationConstraintBase {
    type Target = TransformComponentConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for RotationConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
