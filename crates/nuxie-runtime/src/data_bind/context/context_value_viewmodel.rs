//! Structural child cache owned by C++ `ContextValueViewModel`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::ViewModel(value) => {
            Some(RuntimeDataBindGraphValue::ViewModel(*value))
        }
        _ => None,
    }
}
