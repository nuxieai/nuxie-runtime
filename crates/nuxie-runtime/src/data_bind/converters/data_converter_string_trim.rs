//! Direct owner for pinned C++ `DataConverterStringTrim`.
//!
//! C++ reuses a private `DataValueString m_output` and returns its address.
//! Rust graph values are retained by value, so callers own the returned bytes.

/// Mechanical translation of `trimValue()` and `convert`. `None` represents
/// any non-`DataValueString` input and therefore writes
/// `DataValueString::defaultValue`.
pub(crate) fn convert(value: Option<&[u8]>, trim_type: u64) -> Vec<u8> {
    value
        .map(|value| nuxie_binary::data_converter_string_trim_value(value, trim_type))
        .unwrap_or_default()
}

/// Mechanical translation of the generated `trimType` setter followed by
/// `trimTypeChanged()`. `true` is the Rust graph's dirty signal.
pub(crate) fn set_trim_type(current: &mut u64, value: u64) -> bool {
    if *current == value {
        return false;
    }
    *current = value;
    true
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::RuntimeDataType::String
}
