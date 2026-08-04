use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub fn parse_integer(result: &mut f64, data: &str, base: i32) -> ConstantNumberParseResult {
    LUAU_ASSERT!(base == 2 || base == 16);

    // C++ `strtoull(data, &end, 16)` transparently skips a leading "0x"/"0X"
    // prefix; Rust's `u64::from_str_radix` does NOT (it errors on the 'x'). The
    // binary path strips "0b" before calling in, but the hex path passes the full
    // "0x..." string (as C++ does, relying on strtoull) — so strip it here for
    // base 16, otherwise EVERY hex literal is reported as "Malformed number".
    let digits = if base == 16 && (data.starts_with("0x") || data.starts_with("0X")) {
        &data[2..]
    } else {
        data
    };

    // C++ `strtoull` stops at the first char invalid for the base and parseInteger
    // checks `*end != 0` -> Malformed, BEFORE the overflow check. Rust's
    // from_str_radix hits overflow first on e.g. `0xffff..llllllg` (returning
    // PosOverflow), masking the trailing junk. Validate the digits up front so
    // trailing/invalid characters are Malformed, and a pure-overflow value (all
    // valid digits) is Hex/BinOverflow — matching C++.
    if digits.is_empty() || !digits.bytes().all(|b| (b as char).is_digit(base as u32)) {
        return ConstantNumberParseResult::Malformed;
    }

    match u64::from_str_radix(digits, base as u32) {
        Ok(value) => {
            *result = value as f64;

            // Precision check: doubles have 53 bits of mantissa. If the value is
            // >= 2^53, it might not be representable exactly.
            if value >= (1u64 << 53) && (*result as u64) != value {
                return ConstantNumberParseResult::Imprecise;
            }

            ConstantNumberParseResult::Ok
        }
        Err(e) => {
            // C++ distinguishes overflow (value == ULLONG_MAX && errno == ERANGE)
            // from malformed input (a trailing invalid character).
            if e.kind() == &core::num::IntErrorKind::PosOverflow {
                if base == 2 {
                    ConstantNumberParseResult::BinOverflow
                } else {
                    ConstantNumberParseResult::HexOverflow
                }
            } else {
                ConstantNumberParseResult::Malformed
            }
        }
    }
}
