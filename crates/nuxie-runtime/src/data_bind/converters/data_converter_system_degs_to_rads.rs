//! Direction-aware operation owner matching C++ `DataConverterSystemDegsToRads`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(
    input: &RuntimeDataBindGraphValue,
    operand: f32,
    operation: u64,
    to_source: bool,
) -> f32 {
    if to_source {
        crate::data_converter_operation::reverse(input, operand, operation)
    } else {
        crate::data_converter_operation::convert(input, operand, operation)
    }
}

pub(crate) fn reverse_convert(
    input: &RuntimeDataBindGraphValue,
    operand: f32,
    operation: u64,
    to_target: bool,
) -> f32 {
    if to_target {
        crate::data_converter_operation::convert(input, operand, operation)
    } else {
        crate::data_converter_operation::reverse(input, operand, operation)
    }
}
