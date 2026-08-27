//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_number.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete Number result and its CoreDouble/CoreUint
//! target coercion.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Number(value) => Some(RuntimeDataBindGraphValue::Number(*value)),
        _ => None,
    }
}

/// `calculateValue<DataValueNumber, float>` returns the Number payload, or
/// `DataValueNumber::defaultValue` (zero) for a wrong concrete DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> f32 {
    match value {
        RuntimeDataBindGraphValue::Number(value) => *value,
        _ => 0.0,
    }
}

/// Dynamic/artboard routing reaches this owner before the concrete source
/// subclass is known. Preserve C++'s `DataValueNumber` type test there.
pub(crate) fn number_value(value: &RuntimeDataBindGraphValue) -> Option<f32> {
    match value {
        RuntimeDataBindGraphValue::Number(value) => Some(*value),
        _ => None,
    }
}

/// Defined subset of C++'s `int rounded = value < 0 ? 0 :
/// std::round(value)` before `CoreRegistry::setUint`.
///
/// C++ float-to-int conversion is undefined for NaN, positive infinity, and
/// rounded values outside signed-int range. Rust leaves those inputs
/// unapplied instead of inventing a result; every defined input preserves the
/// exact negative clamp and halfway-away-from-zero rounding.
pub(crate) fn core_uint_value(value: f32) -> Option<u64> {
    if value < 0.0 {
        return Some(0);
    }
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    (rounded < 2_147_483_648.0).then_some(rounded as u64)
}

/// The direct artboard adapter historically receives all Rust `f32` values.
/// Preserve its established saturating-cast behavior for inputs where the C++
/// float-to-int conversion is undefined; defined inputs still use the exact
/// pinned clamp-and-round path above.
pub(crate) fn artboard_core_uint_value(value: f32) -> u64 {
    core_uint_value(value).unwrap_or_else(|| value.round() as u64)
}
