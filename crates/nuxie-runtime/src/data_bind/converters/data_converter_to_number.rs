//! Scalar input ownership matching C++ `DataConverterToNumber`.

pub(crate) fn string(value: &[u8]) -> f32 {
    nuxie_binary::data_converter_to_number_string_value(value)
}
