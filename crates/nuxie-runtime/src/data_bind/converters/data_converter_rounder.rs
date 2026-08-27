//! Direct owner for C++ `DataConverterRounder`.
//!
//! C++ reuses a private `DataValueNumber m_output` and returns its address.
//! Rust data-bind graph values are retained by value, so the caller owns the
//! returned number instead of observing converter-local pointer identity.

pub(crate) fn convert(value: f32, decimals: u64) -> f32 {
    nuxie_binary::data_converter_rounder_value(value, decimals)
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::data_converter_rounder_output_type()
}
