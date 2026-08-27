//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_boolean.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete Boolean result and its CoreBool projection.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::Boolean(value) => {
            Some(RuntimeDataBindGraphValue::Boolean(*value))
        }
        _ => None,
    }
}

/// `calculateValue<DataValueBoolean, bool>` returns the Boolean payload, or
/// `DataValueBoolean::defaultValue` (`false`) for a wrong concrete DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> bool {
    match value {
        RuntimeDataBindGraphValue::Boolean(value) => *value,
        _ => false,
    }
}

/// Dynamic/artboard routing reaches this owner before the concrete source
/// subclass is known. Preserve C++'s `DataValueBoolean` type test there.
pub(crate) fn boolean_value(value: &RuntimeDataBindGraphValue) -> Option<bool> {
    match value {
        RuntimeDataBindGraphValue::Boolean(value) => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_value_uses_cpp_boolean_default_for_wrong_data_value_type() {
        assert!(calculate_value(&RuntimeDataBindGraphValue::Boolean(true)));
        assert!(!calculate_value(&RuntimeDataBindGraphValue::Number(1.0)));
    }

    #[test]
    fn dynamic_projection_rejects_wrong_data_value_type() {
        assert_eq!(
            boolean_value(&RuntimeDataBindGraphValue::Boolean(true)),
            Some(true)
        );
        assert_eq!(boolean_value(&RuntimeDataBindGraphValue::Number(1.0)), None);
    }
}
