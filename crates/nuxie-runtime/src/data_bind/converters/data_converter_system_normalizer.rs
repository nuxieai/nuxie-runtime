//! Direction-aware operation owner matching C++ `DataConverterSystemNormalizer`.

pub(crate) fn reverse(input: f32, operand: f32, operation: u64, to_target: bool) -> f32 {
    if to_target {
        crate::data_converter_operation::convert(input, operand, operation)
    } else {
        crate::data_converter_operation::reverse(input, operand, operation)
    }
}
