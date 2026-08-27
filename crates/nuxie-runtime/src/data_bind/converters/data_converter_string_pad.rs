//! Direct owner for pinned C++ `DataConverterStringPad`.
//!
//! C++ reuses a private `DataValueString m_output` and returns its address.
//! Rust graph values are retained by value, so callers own the returned bytes.

/// Mechanical translation of `convert`. `None` represents any non-string
/// `DataValue` and therefore writes `DataValueString::defaultValue`.
pub(crate) fn convert(
    value: Option<&[u8]>,
    length: u64,
    text: &[u8],
    pad_type: u64,
) -> Vec<u8> {
    value
        .map(|value| nuxie_binary::data_converter_string_pad_value(value, length, text, pad_type))
        .unwrap_or_default()
}

/// Mechanical translation of the generated `length` setter followed by
/// `lengthChanged()`. `true` is the Rust graph's converter-dirty signal.
pub(crate) fn set_length(current: &mut u64, value: u64) -> bool {
    set_uint_property(current, value)
}

/// Mechanical translation of the generated `text` setter followed by
/// `textChanged()`. `true` is the Rust graph's converter-dirty signal.
pub(crate) fn set_text(current: &mut Vec<u8>, value: &[u8]) -> bool {
    if current.as_slice() == value {
        return false;
    }
    *current = value.to_vec();
    true
}

/// Mechanical translation of the generated `padType` setter followed by
/// `padTypeChanged()`. `true` is the Rust graph's converter-dirty signal.
pub(crate) fn set_pad_type(current: &mut u64, value: u64) -> bool {
    set_uint_property(current, value)
}

fn set_uint_property(current: &mut u64, value: u64) -> bool {
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
