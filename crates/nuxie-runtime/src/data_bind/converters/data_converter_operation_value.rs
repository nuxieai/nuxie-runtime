//! Authored-operand owner matching C++ `DataConverterOperationValue`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(input: &RuntimeDataBindGraphValue, operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::convert(input, operand, operation_type)
}

pub(crate) fn reverse(input: &RuntimeDataBindGraphValue, operand: f32, operation_type: u64) -> f32 {
    crate::data_converter_operation::reverse(input, operand, operation_type)
}
