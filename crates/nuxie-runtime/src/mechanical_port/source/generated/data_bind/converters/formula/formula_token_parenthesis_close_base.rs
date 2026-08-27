use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::formula::formula_token_parenthesis::FormulaTokenParenthesis,
    data_bind::converters::formula::formula_token_parenthesis_close::FormulaTokenParenthesisClose,
};

pub struct FormulaTokenParenthesisCloseBase {
    pub base: FormulaTokenParenthesis,
}

impl Default for FormulaTokenParenthesisCloseBase {
    fn default() -> Self {
        Self {
            base: FormulaTokenParenthesis::default(),
        }
    }
}

impl FormulaTokenParenthesisCloseBase {
    pub const TYPE_KEY: u16 = 540;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 539 | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaTokenParenthesisClose {
        let mut cloned = FormulaTokenParenthesisClose::default();
        cloned.base.copy(self);
        cloned
    }
}
