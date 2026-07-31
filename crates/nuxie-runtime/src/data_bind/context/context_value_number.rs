//! Number source compatibility owned by C++ `ContextValueNumber`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Number(value) => Some(RuntimeDataBindGraphValue::Number(*value)),
        _ => None,
    }
}
