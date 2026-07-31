//! Retained-operand owner matching C++ `DataConverterOperationViewModel`.

pub(crate) fn convert(input: f32, retained_operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::convert(input, retained_operand, operation_type)
}

pub(crate) fn reverse(input: f32, retained_operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::reverse(input, retained_operand, operation_type)
}
