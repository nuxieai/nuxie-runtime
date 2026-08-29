use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::data_bind::converters::formula::formula_token_base::FormulaTokenBase,
    importers::{
        data_converter_formula_importer::DataConverterFormulaImporter, import_stack::ImportStack,
    },
    status_code::StatusCode,
};

pub trait DataConverterFormula {
    fn add_token(&mut self, token: CoreHandle);
    fn add_data_bind(&mut self, data_bind: CoreHandle);
}
pub struct FormulaToken {
    pub base: FormulaTokenBase,
    formula: Option<CoreHandle>,
}

impl std::ops::Deref for FormulaToken {
    type Target = FormulaTokenBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FormulaToken {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl Default for FormulaToken {
    fn default() -> Self {
        Self {
            base: FormulaTokenBase::default(),
            formula: None,
        }
    }
}
impl FormulaToken {
    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.handle()
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<DataConverterFormulaImporter>(
            crate::mechanical_port::source::generated::data_bind::converters::data_converter_formula_base::DataConverterFormulaBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(token) = self.handle() else {
            return StatusCode::MissingObject;
        };
        let formula = importer.formula();
        formula.with_downcast_mut::<crate::mechanical_port::source::data_bind::converters::data_converter_formula::DataConverterFormula, _>(|formula| {
            DataConverterFormula::add_token(formula, token);
        });
        self.formula = Some(formula);
        self.base.base.import(stack)
    }
    pub fn add_data_bind(&mut self, data_bind: CoreHandle) {
        if let Some(formula) = self.formula.as_ref() {
            crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(formula.clone()).add_data_bind(data_bind);
        }
    }
}
