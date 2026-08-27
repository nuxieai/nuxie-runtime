//! Direct Rust owner for pinned C++ `FormulaToken`.

use nuxie_binary::RuntimeObject;

/// Import result for the only failure branch owned by `FormulaToken`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeFormulaTokenImportStatus {
    Ok,
    MissingObject,
}

/// The imported token and its retained `DataConverterFormula` owner.
///
/// C++ retains a raw `DataConverterFormula*`; Rust retains the immutable
/// imported object because mutable converter state is instantiated later.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeFormulaToken<'a> {
    object: &'a RuntimeObject,
    formula: Option<&'a RuntimeObject>,
}

impl<'a> RuntimeFormulaToken<'a> {
    /// Mechanical translation of `FormulaToken::import`.
    ///
    /// The binary importer has already performed the latest-importer lookup
    /// and dropped `MissingObject` occurrences. This live runtime seam retains
    /// that resolved formula and appends the token in authored order. Core's
    /// `import` is the final `Ok` represented by this return value.
    pub(crate) fn import(
        formula: Option<&'a RuntimeObject>,
        object: &'a RuntimeObject,
        formula_tokens: &mut Vec<Self>,
    ) -> RuntimeFormulaTokenImportStatus {
        let Some(formula) = formula else {
            return RuntimeFormulaTokenImportStatus::MissingObject;
        };
        formula_tokens.push(Self {
            object,
            formula: Some(formula),
        });
        RuntimeFormulaTokenImportStatus::Ok
    }

    pub(crate) fn object(self) -> &'a RuntimeObject {
        self.object
    }

    /// Mechanical translation of `FormulaToken::addDataBind`.
    ///
    /// `formula_data_binds` is the Rust-owned equivalent of the retained
    /// formula's `m_dataBinds` vector.
    pub(crate) fn add_data_bind<T>(self, formula_data_binds: &mut Vec<T>, data_bind: T) {
        if self.formula.is_some() {
            formula_data_binds.push(data_bind);
        }
    }
}
