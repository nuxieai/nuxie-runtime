use std::any::Any;

use crate::mechanical_port::source::{
    core::CoreHandle, data_bind::converters::data_converter_formula::DataConverterFormula,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct DataConverterFormulaImporter {
    formula: CoreHandle,
}

impl DataConverterFormulaImporter {
    pub fn new(formula: CoreHandle) -> Self {
        Self { formula }
    }

    pub fn formula(&self) -> CoreHandle {
        self.formula.clone()
    }
}

impl ImportStackObject for DataConverterFormulaImporter {
    fn resolve(&mut self) -> StatusCode {
        self.formula
            .with_downcast_mut::<DataConverterFormula, _>(|formula| formula.calculate_formula())
            .expect("DataConverterFormulaImporter retains a DataConverterFormula");
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
