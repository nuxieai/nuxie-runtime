//! Scalar formatting ownership matching C++ `DataConverterToString`.

pub(crate) fn number(value: f32, flags: u64, decimals: u64) -> Vec<u8> {
    nuxie_binary::data_converter_to_string_number_value(value, flags, decimals)
}

pub(crate) fn color(value: u32, format: &[u8]) -> Vec<u8> {
    nuxie_binary::data_converter_to_string_color_value(value, format)
}
