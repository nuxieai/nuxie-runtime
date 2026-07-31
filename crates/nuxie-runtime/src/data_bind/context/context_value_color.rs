//! Color source compatibility owned by C++ `ContextValueColor`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Color(value) => Some(RuntimeDataBindGraphValue::Color(*value)),
        _ => None,
    }
}
