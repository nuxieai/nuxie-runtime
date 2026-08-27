use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::formula::formula_token::FormulaToken,
    data_bind::converters::formula::formula_token_parenthesis::FormulaTokenParenthesis,
};

pub struct FormulaTokenParenthesisBase {
    pub base: FormulaToken,
}

impl Default for FormulaTokenParenthesisBase {
    fn default() -> Self {
        Self {
            base: FormulaToken::default(),
        }
    }
}

impl FormulaTokenParenthesisBase {
    pub const TYPE_KEY: u16 = 539;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaTokenParenthesis {
        let mut cloned = FormulaTokenParenthesis::default();
        cloned.base.copy(self);
        cloned
    }
}
