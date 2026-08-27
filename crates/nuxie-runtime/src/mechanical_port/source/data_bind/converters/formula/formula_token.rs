#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusCode {
    Ok,
    MissingObject,
}
pub trait DataBind {}
pub trait DataConverterFormula {
    fn add_token(&mut self, token: *mut FormulaToken);
    fn add_data_bind(&mut self, data_bind: *mut dyn DataBind);
}
pub trait FormulaImportStack {
    fn latest_formula(&mut self) -> Option<&mut dyn DataConverterFormula>;
    fn import_formula_token_super(&mut self, token: &mut FormulaToken) -> StatusCode;
}
pub struct FormulaToken {
    formula: Option<*mut dyn DataConverterFormula>,
}
impl Default for FormulaToken {
    fn default() -> Self {
        Self { formula: None }
    }
}
impl FormulaToken {
    pub fn import(&mut self, stack: &mut dyn FormulaImportStack) -> StatusCode {
        let Some(formula) = stack.latest_formula() else {
            return StatusCode::MissingObject;
        };
        let formula_ptr = formula as *mut dyn DataConverterFormula;
        formula.add_token(self as *mut Self);
        self.formula = Some(formula_ptr);
        stack.import_formula_token_super(self)
    }
    pub fn add_data_bind(&mut self, data_bind: *mut dyn DataBind) {
        if let Some(formula) = self.formula {
            unsafe {
                (&mut *formula).add_data_bind(data_bind);
            }
        }
    }
}
