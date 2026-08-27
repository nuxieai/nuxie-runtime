//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_string.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete String result and its CoreString projection.

use nuxie_binary::RuntimeDataType;

use crate::data_bind_graph::{RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue};

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::String(value) => {
            Some(RuntimeDataBindGraphValue::String(value.clone()))
        }
        _ => None,
    }
}

/// Whether pinned `DataBind::bind` selects `DataBindContextValueString` for
/// this occurrence. A concrete converter output wins; `none` and `input`
/// fall back to the live source type, while `any` stays dynamically typed.
pub(crate) fn owns_output(
    source: &RuntimeDataBindGraphValue,
    converter: Option<&RuntimeDataBindGraphConverter>,
) -> bool {
    match converter.map(RuntimeDataBindGraphConverter::cpp_output_data_type) {
        Some(RuntimeDataType::String) => true,
        None | Some(RuntimeDataType::None | RuntimeDataType::Input) => {
            matches!(source, RuntimeDataBindGraphValue::String(_))
        }
        _ => false,
    }
}

/// `calculateValue<DataValueString, std::string>` returns the String payload,
/// or `DataValueString::defaultValue` (the empty string) for a wrong concrete
/// DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> Vec<u8> {
    match value {
        RuntimeDataBindGraphValue::String(value) => value.clone(),
        _ => Vec::new(),
    }
}

/// Dynamic/artboard routing reaches this owner before the concrete source
/// subclass is known. Preserve C++'s `DataValueString` type test there.
pub(crate) fn string_value(value: &RuntimeDataBindGraphValue) -> Option<&[u8]> {
    match value {
        RuntimeDataBindGraphValue::String(value) => Some(value),
        _ => None,
    }
}
