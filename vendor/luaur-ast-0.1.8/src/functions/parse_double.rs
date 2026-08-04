use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use crate::functions::parse_integer::parse_integer;

pub fn parse_double(result: &mut f64, data: &str) -> ConstantNumberParseResult {
    let bytes = data.as_bytes();

    // binary literal
    if bytes.len() >= 3 && bytes[0] == b'0' && (bytes[1] == b'b' || bytes[1] == b'B') {
        return parse_integer(result, &data[2..], 2);
    }

    // hexadecimal literal
    if bytes.len() >= 3 && bytes[0] == b'0' && (bytes[1] == b'x' || bytes[1] == b'X') {
        // pass in '0x' prefix, it's handled by 'strtoull' in C++ / parse_integer in Rust
        return parse_integer(result, data, 16);
    }

    // Rust's f64::from_str is the equivalent of strtod.
    // However, strtod stops at the first invalid character and returns the end pointer.
    // f64::from_str requires the entire string to be valid.
    let value = match data.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return ConstantNumberParseResult::Malformed,
    };

    *result = value;

    // for linting, we detect integer constants that are parsed imprecisely
    // since the check is expensive we only perform it when the number is larger than the precise integer range
    if value >= (1u64 << 53) as f64 && data.chars().all(|c| c.is_ascii_digit()) {
        // Equivalent to snprintf(repr, sizeof(repr), "%.0f", value);
        let repr = format!("{:.0}", value);

        if repr != data {
            return ConstantNumberParseResult::Imprecise;
        }
    }

    ConstantNumberParseResult::Ok
}
