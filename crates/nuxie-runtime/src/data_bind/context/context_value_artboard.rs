//! Integer-backed source compatibility owned by C++ `ContextValueArtboard`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    super_integer(next).map(RuntimeDataBindGraphValue::Artboard)
}

fn super_integer(value: &RuntimeDataBindGraphValue) -> Option<u64> {
    crate::context_value_enum::integer_payload(value)
}
