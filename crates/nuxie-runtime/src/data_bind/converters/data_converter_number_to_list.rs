//! Direct owner for C++ `DataConverterNumberToList`.

pub(crate) fn convert(value: f32, view_model_exists: bool) -> usize {
    if view_model_exists {
        crate::project_data_converter::project_data_converter_bounded_list_length(f64::from(value))
    } else {
        0
    }
}
