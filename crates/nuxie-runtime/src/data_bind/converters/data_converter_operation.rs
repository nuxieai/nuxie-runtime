//! Scalar operation semantics shared by C++ operation converters.

pub(crate) fn positive_mod(value: f32, mut range: f32) -> f32 {
    if range < 0.0 {
        range = -range;
    }
    let mut value = value % range;
    if value < 0.0 {
        value += range;
    }
    value
}

pub(crate) fn convert(input: f32, operand: f32, operation_type: u64) -> f32 {
    match operation_type {
        0 => input + operand,
        1 => input - operand,
        2 => input * operand,
        3 => input / operand,
        4 => positive_mod(input, operand),
        5 => input.sqrt(),
        6 => input.powf(operand),
        7 => input.exp(),
        8 => input.ln(),
        9 => input.cos(),
        10 => input.sin(),
        11 => input.tan(),
        12 => input.acos(),
        13 => input.asin(),
        14 => input.atan(),
        15 => input.atan2(operand),
        16 => input.round(),
        17 => input.floor(),
        18 => input.ceil(),
        _ => operand,
    }
}

pub(crate) fn reverse(input: f32, operand: f32, operation_type: u64) -> f32 {
    match operation_type {
        0 => input - operand,
        1 => input + operand,
        2 => input / operand,
        3 => input * operand,
        4 => input,
        5 => input.powf(2.0),
        6 => input.powf(1.0 / operand),
        7 => input.ln(),
        8 => input.exp(),
        9 => input.acos(),
        10 => input.asin(),
        11 => input.atan(),
        12 => input.cos(),
        13 => input.sin(),
        14 => input.tan(),
        15..=18 => input,
        _ => operand,
    }
}
