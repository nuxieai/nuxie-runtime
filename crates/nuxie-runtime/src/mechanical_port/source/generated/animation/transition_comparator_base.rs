use crate::mechanical_port::source::{core::binary_reader::BinaryReader, core::Core};

pub struct TransitionComparatorBase {
    pub base: Core,
}

impl Default for TransitionComparatorBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
        }
    }
}

impl TransitionComparatorBase {
    pub const TYPE_KEY: u16 = 477;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {}
    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        false
    }
}

impl std::ops::Deref for TransitionComparatorBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
