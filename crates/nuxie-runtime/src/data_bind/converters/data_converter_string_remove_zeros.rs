//! Direct owner for pinned C++ `DataConverterStringRemoveZeros`.
//!
//! C++ reuses a private `DataValueString m_output` and returns its address.
//! Rust graph values are retained by value, so callers own the returned bytes.

/// Mechanical translation of the static `removeZeros` helper.
pub(crate) fn remove_zeros(value: &[u8]) -> Vec<u8> {
    nuxie_binary::data_converter_string_remove_zeros_value(value)
}

/// Mechanical translation of `convert`. `None` represents any non-string
/// `DataValue` and therefore writes `DataValueString::defaultValue`.
pub(crate) fn convert(value: Option<&[u8]>) -> Vec<u8> {
    value.map(remove_zeros).unwrap_or_default()
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::RuntimeDataType::String
}
