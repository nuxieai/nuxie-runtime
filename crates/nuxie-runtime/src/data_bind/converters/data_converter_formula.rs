//! Formula stack ownership matching C++ `DataConverterFormula`.

pub(crate) fn binary_operation(left: f32, right: f32, operation_type: u64) -> f32 {
    match operation_type {
        0 => left + right,
        1 => left - right,
        2 => left * right,
        3 => left / right,
        4 => crate::data_converter_operation::positive_mod(left, right),
        _ => 0.0,
    }
}
