//! Integer-backed source compatibility owned by C++ `ContextValueEnum`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn integer_payload(value: &RuntimeDataBindGraphValue) -> Option<u64> {
    match value {
        RuntimeDataBindGraphValue::Integer(value)
        | RuntimeDataBindGraphValue::Enum(value)
        | RuntimeDataBindGraphValue::SymbolListIndex(value)
        | RuntimeDataBindGraphValue::Asset(value)
        | RuntimeDataBindGraphValue::Artboard(value)
        | RuntimeDataBindGraphValue::Trigger(value) => Some(*value),
        _ => None,
    }
}

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    integer_payload(next).map(RuntimeDataBindGraphValue::Enum)
}
