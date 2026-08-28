use crate::mechanical_port::source::generated::data_bind::converters::formula::formula_token_parenthesis_base::FormulaTokenParenthesisBase;

#[derive(Default)]
pub struct FormulaTokenParenthesis {
    pub base: FormulaTokenParenthesisBase,
}

impl std::ops::Deref for FormulaTokenParenthesis {
    type Target = FormulaTokenParenthesisBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FormulaTokenParenthesis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
