use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
        data_value_color::DataValueColor, data_value_enum::DataValueEnum,
        data_value_number::DataValueNumber, data_value_string::DataValueString,
        data_value_symbol_list_index::DataValueSymbolListIndex,
    },
    generated::data_bind::converters::data_converter_to_number_base::DataConverterToNumberBase,
};
#[derive(Default)]
pub struct DataConverterToNumber {
    pub base: DataConverterToNumberBase,
    output: DataValueNumber,
}
impl DataConverterToNumber {
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let value = if let Some(value) = input.as_any().downcast_ref::<DataValueString>() {
            // Keep the pinned `std::atof` prefix/range semantics without a C
            // runtime dependency. The parser lives with this owner, so the
            // translated path never calls the superseded binary runtime.
            convert_string(value.value().as_bytes(), self.output.value())
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueEnum>() {
            value.value() as f32
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueNumber>() {
            value.value()
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueColor>() {
            value.value() as f32
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueBoolean>() {
            if value.value() { 1.0 } else { 0.0 }
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueSymbolListIndex>() {
            value.value() as f32
        } else {
            DataValueNumber::DEFAULT_VALUE
        };
        self.output.set_value(value);
        &self.output
    }
}

crate::impl_data_converter_capability_forward!(DataConverterToNumber, base.base);

fn convert_string(value: &[u8], previous_output: f32) -> f32 {
    let parsed = cpp_atof_f32(value);
    if parsed.range_error {
        previous_output
    } else {
        parsed.value
    }
}

#[derive(Clone, Copy)]
struct CppAtofF32 {
    value: f32,
    range_error: bool,
}

fn cpp_atof_f32(value: &[u8]) -> CppAtofF32 {
    let mut start = 0usize;
    while start < value.len() && value[start].is_ascii_whitespace() {
        start += 1;
    }

    let mut number_start = start;
    let sign = match value.get(number_start) {
        Some(b'-') => {
            number_start += 1;
            -1.0
        }
        Some(b'+') => {
            number_start += 1;
            1.0
        }
        _ => 1.0,
    };
    if value.get(number_start) == Some(&b'0')
        && value
            .get(number_start + 1)
            .is_some_and(|byte| matches!(*byte, b'x' | b'X'))
    {
        return cpp_atof_hex_f32(value, number_start, sign);
    }

    let keyword = &value[number_start..];
    if keyword
        .get(..8)
        .is_some_and(|value| value.eq_ignore_ascii_case(b"infinity"))
        || keyword
            .get(..3)
            .is_some_and(|value| value.eq_ignore_ascii_case(b"inf"))
    {
        return CppAtofF32 {
            value: (sign as f32) * f32::INFINITY,
            range_error: false,
        };
    }
    if keyword
        .get(..3)
        .is_some_and(|value| value.eq_ignore_ascii_case(b"nan"))
    {
        return CppAtofF32 {
            value: f32::NAN.copysign(sign as f32),
            range_error: false,
        };
    }

    let mut end = number_start;
    let mut digits = 0usize;
    let mut nonzero_digit = false;
    while value.get(end).is_some_and(u8::is_ascii_digit) {
        nonzero_digit |= value[end] != b'0';
        end += 1;
        digits += 1;
    }

    if value.get(end) == Some(&b'.') {
        end += 1;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            nonzero_digit |= value[end] != b'0';
            end += 1;
            digits += 1;
        }
    }

    if digits == 0 {
        return CppAtofF32 {
            value: 0.0,
            range_error: false,
        };
    }

    let mantissa_end = end;
    if value
        .get(end)
        .is_some_and(|byte| matches!(*byte, b'e' | b'E'))
    {
        end += 1;
        if value
            .get(end)
            .is_some_and(|byte| matches!(*byte, b'+' | b'-'))
        {
            end += 1;
        }
        let exponent_start = end;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if exponent_start == end {
            end = mantissa_end;
        }
    }

    let Ok(parsed) = std::str::from_utf8(&value[start..end])
        .expect("numeric prefix is ASCII")
        .parse::<f64>()
    else {
        return CppAtofF32 {
            value: 0.0,
            range_error: true,
        };
    };

    CppAtofF32 {
        value: parsed as f32,
        range_error: !parsed.is_finite()
            || (nonzero_digit && parsed == 0.0)
            || (parsed != 0.0 && parsed.abs() < f64::MIN_POSITIVE),
    }
}

fn cpp_atof_hex_f32(value: &[u8], number_start: usize, sign: f64) -> CppAtofF32 {
    let mut end = number_start + 2;
    let mut digits = 0usize;
    let mut fraction_digits = 0usize;
    let mut after_point = false;
    let mut significant_bits = 0usize;
    let mut prefix = 0u64;
    let mut prefix_bits = 0usize;
    let mut tail_nonzero = false;

    loop {
        if !after_point && value.get(end) == Some(&b'.') {
            after_point = true;
            end += 1;
            continue;
        }
        let Some(digit) = value.get(end).and_then(|byte| ascii_hex_digit_value(*byte)) else {
            break;
        };
        digits += 1;
        fraction_digits += usize::from(after_point);
        end += 1;

        let width = if significant_bits == 0 {
            if digit == 0 {
                continue;
            }
            (u8::BITS - digit.leading_zeros()) as usize
        } else {
            4
        };
        significant_bits += width;
        for shift in (0..width).rev() {
            let bit = (digit >> shift) & 1;
            if prefix_bits < u64::BITS as usize {
                prefix = (prefix << 1) | u64::from(bit);
                prefix_bits += 1;
            } else {
                tail_nonzero |= bit != 0;
            }
        }
    }

    if digits == 0 || significant_bits == 0 {
        return CppAtofF32 {
            value: 0.0f32.copysign(sign as f32),
            range_error: false,
        };
    }

    let mut exponent = 0i128;
    if value
        .get(end)
        .is_some_and(|byte| matches!(*byte, b'p' | b'P'))
    {
        end += 1;
        let exponent_sign = match value.get(end) {
            Some(b'-') => {
                end += 1;
                -1
            }
            Some(b'+') => {
                end += 1;
                1
            }
            _ => 1,
        };
        let exponent_start = end;
        let mut exponent_value = 0i128;
        while let Some(digit) = value
            .get(end)
            .filter(|byte| byte.is_ascii_digit())
            .map(|byte| i128::from(*byte - b'0'))
        {
            exponent_value = exponent_value.saturating_mul(10).saturating_add(digit);
            end += 1;
        }
        if exponent_start != end {
            exponent = i128::from(exponent_sign) * exponent_value;
        }
    }

    if prefix_bits < u64::BITS as usize {
        prefix <<= u64::BITS as usize - prefix_bits;
    }

    let mut unbiased_exponent = exponent
        .saturating_add(significant_bits as i128 - 1)
        .saturating_sub((fraction_digits as i128).saturating_mul(4));
    if unbiased_exponent > 1023 {
        return CppAtofF32 {
            value: (sign as f32) * f32::INFINITY,
            range_error: true,
        };
    }

    let retained_bits = if unbiased_exponent >= -1022 {
        53
    } else {
        unbiased_exponent + 1075
    };
    if retained_bits < 0 || retained_bits == 0 && prefix == 1u64 << 63 && !tail_nonzero {
        return CppAtofF32 {
            value: 0.0f32.copysign(sign as f32),
            range_error: true,
        };
    }

    let retained_bits = retained_bits as usize;
    let mut retained = if retained_bits == 0 {
        0
    } else {
        prefix >> (u64::BITS as usize - retained_bits)
    };
    let guard = (prefix >> (u64::BITS as usize - retained_bits - 1)) & 1 != 0;
    let remaining_prefix_bits = u64::BITS as usize - retained_bits - 1;
    let remaining_nonzero = remaining_prefix_bits != 0
        && prefix & ((1u64 << remaining_prefix_bits) - 1) != 0
        || tail_nonzero;
    if guard && (remaining_nonzero || retained & 1 != 0) {
        retained += 1;
    }

    if unbiased_exponent >= -1022 && retained == 1u64 << 53 {
        retained >>= 1;
        unbiased_exponent += 1;
        if unbiased_exponent > 1023 {
            return CppAtofF32 {
                value: (sign as f32) * f32::INFINITY,
                range_error: true,
            };
        }
    }

    let sign_bit = u64::from(sign.is_sign_negative()) << 63;
    let bits = if unbiased_exponent >= -1022 {
        sign_bit | ((unbiased_exponent as u64 + 1023) << 52) | (retained & ((1u64 << 52) - 1))
    } else {
        sign_bit | retained
    };
    let parsed = f64::from_bits(bits);
    CppAtofF32 {
        value: parsed as f32,
        range_error: bits & (0x7ffu64 << 52) == 0,
    }
}

fn ascii_hex_digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
