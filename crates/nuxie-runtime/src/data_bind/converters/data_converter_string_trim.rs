//! Direct owner for C++ `DataConverterStringTrim`.

pub(crate) fn convert(value: &[u8], trim_type: u64) -> Vec<u8> {
    nuxie_binary::data_converter_string_trim_value(value, trim_type)
}
