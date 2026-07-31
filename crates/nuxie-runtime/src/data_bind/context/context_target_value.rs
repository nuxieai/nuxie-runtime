//! Target-side typed cache owned by C++ `ContextTargetValue`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn changed(
    current: &RuntimeDataBindGraphValue,
    next: &RuntimeDataBindGraphValue,
) -> bool {
    current != next
}
