//! Retained-operand owner matching C++ `DataConverterOperationViewModel`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(
    input: &RuntimeDataBindGraphValue,
    retained_operand: f32,
    operation_type: u64,
) -> f32 {
    crate::data_converter_operation::convert(input, retained_operand, operation_type)
}

pub(crate) fn reverse(
    input: &RuntimeDataBindGraphValue,
    retained_operand: f32,
    operation_type: u64,
) -> f32 {
    crate::data_converter_operation::reverse(input, retained_operand, operation_type)
}
