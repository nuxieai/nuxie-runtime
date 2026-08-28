use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::formula::formula_token_parenthesis::FormulaTokenParenthesis,
    data_bind::converters::formula::formula_token_parenthesis_open::FormulaTokenParenthesisOpen,
};

pub struct FormulaTokenParenthesisOpenBase {
    pub base: FormulaTokenParenthesis,
}

impl Default for FormulaTokenParenthesisOpenBase {
    fn default() -> Self {
        Self {
            base: FormulaTokenParenthesis::default(),
        }
    }
}

impl FormulaTokenParenthesisOpenBase {
    pub const TYPE_KEY: u16 = 544;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 539 | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> FormulaTokenParenthesisOpen {
        let mut cloned = FormulaTokenParenthesisOpen::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for FormulaTokenParenthesisOpenBase {
    type Target = FormulaTokenParenthesis;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FormulaTokenParenthesisOpenBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
