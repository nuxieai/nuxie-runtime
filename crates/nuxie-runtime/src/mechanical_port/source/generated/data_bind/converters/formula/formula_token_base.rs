use crate::mechanical_port::source::{
    core::Core, core::binary_reader::BinaryReader,
    data_bind::converters::formula::formula_token::FormulaToken,
};

pub struct FormulaTokenBase {
    pub base: Core,
}

impl Default for FormulaTokenBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
        }
    }
}

impl FormulaTokenBase {
    pub const TYPE_KEY: u16 = 537;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaToken {
        let mut cloned = FormulaToken::default();
        cloned.base.copy(self);
        cloned
    }
    pub fn copy(&mut self, object: &Self) {}
    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        false
    }
}
