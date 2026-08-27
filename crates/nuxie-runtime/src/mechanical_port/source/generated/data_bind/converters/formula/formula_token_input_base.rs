use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::formula::formula_token::FormulaToken,
    data_bind::converters::formula::formula_token_input::FormulaTokenInput,
};

pub struct FormulaTokenInputBase {
    pub base: FormulaToken,
}

impl Default for FormulaTokenInputBase {
    fn default() -> Self {
        Self {
            base: FormulaToken::default(),
        }
    }
}

impl FormulaTokenInputBase {
    pub const TYPE_KEY: u16 = 545;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaTokenInput {
        let mut cloned = FormulaTokenInput::default();
        cloned.base.copy(self);
        cloned
    }
}
