use crate::mechanical_port::source::{
    constraints::transform_component_constraint_y::TransformComponentConstraintY,
    constraints::translation_constraint::TranslationConstraint, core::binary_reader::BinaryReader,
};

pub struct TranslationConstraintBase {
    pub base: TransformComponentConstraintY,
}

impl Default for TranslationConstraintBase {
    fn default() -> Self {
        Self {
            base: TransformComponentConstraintY::default(),
        }
    }
}

impl TranslationConstraintBase {
    pub const TYPE_KEY: u16 = 87;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 86 | 85 | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TranslationConstraint {
        let mut cloned = TranslationConstraint::default();
        let mut callbacks = TranslationConstraint::default();
        cloned.base.copy(self, &mut callbacks);
        cloned
    }
}

impl std::ops::Deref for TranslationConstraintBase {
    type Target = TransformComponentConstraintY;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TranslationConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
