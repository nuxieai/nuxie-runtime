use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::formula::formula_token::FormulaToken,
    data_bind::converters::formula::formula_token_argument_separator::FormulaTokenArgumentSeparator,
};

pub struct FormulaTokenArgumentSeparatorBase {
    pub base: FormulaToken,
}

impl Default for FormulaTokenArgumentSeparatorBase {
    fn default() -> Self {
        Self {
            base: FormulaToken::default(),
        }
    }
}

impl FormulaTokenArgumentSeparatorBase {
    pub const TYPE_KEY: u16 = 538;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaTokenArgumentSeparator {
        let mut cloned = FormulaTokenArgumentSeparator::default();
        cloned.base.copy(self);
        cloned
    }
}
