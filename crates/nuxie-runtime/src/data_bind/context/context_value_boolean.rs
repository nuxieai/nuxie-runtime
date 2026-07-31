//! Boolean source compatibility owned by C++ `ContextValueBoolean`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Boolean(value) => {
            Some(RuntimeDataBindGraphValue::Boolean(*value))
        }
        _ => None,
    }
}
