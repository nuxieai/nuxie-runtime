use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, inputs::user_input::UserInput,
};

#[derive(Default)]
pub struct UserInputBase;

impl UserInputBase {
    pub const TYPE_KEY: u16 = 663;

    pub fn is_type_of(type_key: u16) -> bool {
        type_key == Self::TYPE_KEY
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn clone_into(&self) -> UserInput {
        let mut cloned = UserInput::default();
        cloned.base.copy(self);
        cloned
    }

    pub fn copy(&mut self, _object: &Self) {}

    pub fn deserialize(&mut self, _property_key: u16, _reader: &mut BinaryReader<'_>) -> bool {
        false
    }
}
