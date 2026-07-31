//! Direct owner for C++ `DataConverterStringPad`.

pub(crate) fn convert(value: &[u8], length: u64, text: &[u8], pad_type: u64) -> Vec<u8> {
    nuxie_binary::data_converter_string_pad_value(value, length, text, pad_type)
}
