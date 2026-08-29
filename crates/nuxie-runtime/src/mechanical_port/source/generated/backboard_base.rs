use crate::mechanical_port::source::{
    backboard::Backboard,
    core::{Core, binary_reader::BinaryReader},
};

#[derive(Default)]
pub struct BackboardBase {
    pub base: Core,
}

impl BackboardBase {
    pub const TYPE_KEY: u16 = 23;
    pub fn is_type_of(type_key: u16) -> bool {
        type_key == Self::TYPE_KEY
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, _object: &Self) {}
    pub fn deserialize(&mut self, _property_key: u16, _reader: &mut BinaryReader<'_>) -> bool {
        false
    }
    pub fn clone_into(&self) -> Backboard {
        let mut cloned = Backboard::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for BackboardBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BackboardBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
