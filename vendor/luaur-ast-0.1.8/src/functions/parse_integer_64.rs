use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub fn parse_integer_64(result: &mut i64, data: &str, base: i32) -> ConstantNumberParseResult {
    LUAU_ASSERT!(base == 2 || base == 10 || base == 16);

    // We use a helper to simulate C strtoll/strtoull behavior on a substring.
    // The C++ code expects the string to end with 'i\0' after the number.
    if base == 10 {
        // Find the 'i' suffix
        let i_pos = data.find('i');
        if i_pos.is_none()
            || i_pos.unwrap() == 0
            || data.get(i_pos.unwrap() + 1..).unwrap_or("") != ""
        {
            return ConstantNumberParseResult::Malformed;
        }

        let num_str = &data[..i_pos.unwrap()];
        match i64::from_str_radix(num_str, 10) {
            Ok(val) => {
                *result = val;
                ConstantNumberParseResult::Ok
            }
            Err(e) => {
                if e.kind() == &core::num::IntErrorKind::PosOverflow
                    || e.kind() == &core::num::IntErrorKind::NegOverflow
                {
                    ConstantNumberParseResult::IntOverflow
                } else {
                    ConstantNumberParseResult::Malformed
                }
            }
        }
    } else {
        // hex and binary literals represent bit patterns covering the full uint64 range
        let i_pos = data.find('i');
        if i_pos.is_none()
            || i_pos.unwrap() == 0
            || data.get(i_pos.unwrap() + 1..).unwrap_or("") != ""
        {
            return ConstantNumberParseResult::Malformed;
        }

        let num_str = &data[..i_pos.unwrap()];
        match u64::from_str_radix(num_str, base as u32) {
            Ok(u) => {
                *result = u as i64;
                ConstantNumberParseResult::Ok
            }
            Err(e) => {
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
}
