use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub struct TransitionConditionBase {
    pub base: Core,
}

impl Default for TransitionConditionBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
        }
    }
}

impl TransitionConditionBase {
    pub const TYPE_KEY: u16 = 476;

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

impl std::ops::Deref for TransitionConditionBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionConditionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
