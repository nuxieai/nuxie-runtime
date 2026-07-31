//! Symbol-list index source compatibility owned by C++ `ContextValueSymbolListIndex`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::context_value_enum::integer_payload(next).map(RuntimeDataBindGraphValue::SymbolListIndex)
}
