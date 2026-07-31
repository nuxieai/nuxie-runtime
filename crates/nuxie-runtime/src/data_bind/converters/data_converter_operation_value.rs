//! Authored-operand owner matching C++ `DataConverterOperationValue`.

pub(crate) fn convert(input: f32, operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::convert(input, operand, operation_type)
}

pub(crate) fn reverse(input: f32, operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::reverse(input, operand, operation_type)
}
