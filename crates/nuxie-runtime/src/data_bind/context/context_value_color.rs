//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_color.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete Color result and its CoreColor projection.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Color(value) => Some(RuntimeDataBindGraphValue::Color(*value)),
        _ => None,
    }
}

/// `calculateValue<DataValueColor, int>` returns the Color payload, or
/// `DataValueColor::defaultValue` (zero) for a wrong concrete DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> u32 {
    match value {
        RuntimeDataBindGraphValue::Color(value) => *value,
        _ => 0,
    }
}

/// Dynamic/artboard routing reaches this owner before the concrete source
/// subclass is known. Preserve C++'s `DataValueColor` type test there.
pub(crate) fn color_value(value: &RuntimeDataBindGraphValue) -> Option<u32> {
    match value {
        RuntimeDataBindGraphValue::Color(value) => Some(*value),
        _ => None,
    }
}
