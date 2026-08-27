//! Direct owner for C++ `DataConverterBooleanNegate`.
//!
//! C++ reuses a private `DataValueBoolean m_output` and returns its address.
//! Rust data-bind graph values are retained by value, so callers own the
//! returned boolean rather than observing converter-local pointer identity.

pub(crate) fn convert(value: Option<bool>) -> bool {
    nuxie_binary::data_converter_boolean_negate_value(value)
}

/// Mechanical translation of `reverseConvert`: delegate to `convert`.
pub(crate) fn reverse_convert(value: Option<bool>) -> bool {
    convert(value)
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::data_converter_boolean_negate_output_type()
}
