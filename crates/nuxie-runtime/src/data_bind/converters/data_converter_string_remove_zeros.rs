//! Direct owner for C++ `DataConverterStringRemoveZeros`.

pub(crate) fn convert(value: &[u8]) -> Vec<u8> {
    nuxie_binary::data_converter_string_remove_zeros_value(value)
}
