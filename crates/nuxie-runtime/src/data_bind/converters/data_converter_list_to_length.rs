//! Direct owner for C++ `DataConverterListToLength`.

pub(crate) fn convert(item_count: Option<usize>) -> f32 {
    nuxie_binary::data_converter_list_to_length_value(item_count)
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::data_converter_list_to_length_output_type()
}
