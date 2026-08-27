use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    data_bind::converters::data_converter_formula::DataConverterFormula, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct DataConverterFormulaImporter {
    formula: NonNull<DataConverterFormula>,
}

impl DataConverterFormulaImporter {
    pub fn new(formula: NonNull<DataConverterFormula>) -> Self {
        Self { formula }
    }

    pub fn formula(&self) -> NonNull<DataConverterFormula> {
        self.formula
    }
}

impl ImportStackObject for DataConverterFormulaImporter {
    fn resolve(&mut self) -> StatusCode {
        unsafe { self.formula.as_mut().calculate_formula() };
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
