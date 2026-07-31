//! Runtime token classification owned by C++ `FormulaToken`.

use crate::data_bind_graph::RuntimeDataBindGraphFormulaToken;

pub(crate) fn is_source_change_random(token: &RuntimeDataBindGraphFormulaToken) -> bool {
    matches!(
        token,
        RuntimeDataBindGraphFormulaToken::Function {
            function_type: 16,
            random_mode: 2,
            ..
        }
    )
}
