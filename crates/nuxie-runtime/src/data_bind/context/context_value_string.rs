//! String source compatibility owned by C++ `ContextValueString`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::String(value) => {
            Some(RuntimeDataBindGraphValue::String(value.clone()))
        }
        _ => None,
    }
}
