//! Direct owner for pinned C++ `DataConverterToString`.
//!
//! C++ reuses one private `DataValueString m_output`. Rust graph values are
//! retained by value, so callers own the returned bytes. The optional enum
//! names are the Rust projection of `DataValueEnum::dataEnum()`. An
//! out-of-range value produces the pinned empty enum display value; callers
//! that cannot retain the enum metadata currently supply no names and remain
//! an explicit source-correction gap.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(
    input: &RuntimeDataBindGraphValue,
    flags: u64,
    decimals: u64,
    color_format: &[u8],
    enum_value_names: Option<&[Vec<u8>]>,
) -> RuntimeDataBindGraphValue {
    RuntimeDataBindGraphValue::String(match input {
        RuntimeDataBindGraphValue::Number(value) => number(*value, flags, decimals),
        RuntimeDataBindGraphValue::Enum(value) => enum_value_names
            .and_then(|names| names.get((*value as u32) as usize))
            .cloned()
            .unwrap_or_default(),
        RuntimeDataBindGraphValue::String(value) => string(value),
        RuntimeDataBindGraphValue::Color(value) => color(*value, color_format),
        RuntimeDataBindGraphValue::Boolean(value) => boolean(*value),
        RuntimeDataBindGraphValue::Trigger(value) => integer(*value),
        RuntimeDataBindGraphValue::SymbolListIndex(value) => integer(*value),
        _ => Vec::new(),
    })
}

/// `DataConverterToString` does not override `DataConverter::reverseConvert`.
pub(crate) fn reverse_convert(input: &RuntimeDataBindGraphValue) -> RuntimeDataBindGraphValue {
    input.clone()
}

fn number(value: f32, flags: u64, decimals: u64) -> Vec<u8> {
    // Both serialized properties are CoreUint values in C++. Preserve their
    // uint32_t assignment before applying the converter flags/precision.
    nuxie_binary::data_converter_to_string_number_value(
        value,
        u64::from(flags as u32),
        u64::from(decimals as u32),
    )
}

fn color(value: u32, format: &[u8]) -> Vec<u8> {
    nuxie_binary::data_converter_to_string_color_value(value, format)
}

fn boolean(value: bool) -> Vec<u8> {
    if value { b"1".to_vec() } else { b"0".to_vec() }
}

fn string(value: &[u8]) -> Vec<u8> {
    value.to_vec()
}

fn integer(value: u64) -> Vec<u8> {
    (value as u32).to_string().into_bytes()
}

pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::RuntimeDataType::String
}
