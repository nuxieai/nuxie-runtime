//! Pinned `tests/unit_tests/runtime/math_test.cpp` against translated owners.

use math_types::{
    cmp_equal, cmp_greater, cmp_greater_equal, cmp_less, cmp_less_equal, cmp_not_equal,
    positive_mod,
};
use nuxie_runtime::source::math::{bitwise, math_types};

#[test]
fn translated_constants_preserve_upstream_f32_bits() {
    assert_eq!(math_types::PI.to_bits(), 0x4049_0fdb);
    assert_eq!(math_types::SQRT_2.to_bits(), 0x3fb5_04f3);
}

#[test]
fn same_type_lossless_float_casts_reject_nan() {
    assert!(
        std::panic::catch_unwind(|| math_types::lossless_numeric_cast::<f32, _>(f32::NAN)).is_err()
    );
    assert!(
        std::panic::catch_unwind(|| math_types::lossless_numeric_cast::<f64, _>(f64::NAN)).is_err()
    );
}

#[test]
fn ieee_float_divide() {
    let infinity = f32::INFINITY;
    let nan = f32::NAN;
    // Upstream spells this `-0`; unary minus is applied to integer zero before
    // conversion to float, so it is positive IEEE zero rather than `-0.0f`.
    let negative_integer_zero = (-0i32) as f32;
    assert_eq!(math_types::ieee_float_divide(100.0, 10.0), 10.0);
    assert_eq!(math_types::ieee_float_divide(5.0, 0.0), infinity);
    assert_eq!(
        math_types::ieee_float_divide(5.0, negative_integer_zero),
        infinity
    );
    assert_eq!(math_types::ieee_float_divide(-3.0, 0.0), -infinity);
    assert_eq!(
        math_types::ieee_float_divide(-3.0, negative_integer_zero),
        -infinity
    );
    assert_eq!(math_types::ieee_float_divide(infinity, 0.0), infinity);
    assert_eq!(math_types::ieee_float_divide(-infinity, 0.0), -infinity);
    assert_eq!(
        math_types::ieee_float_divide(infinity, negative_integer_zero),
        infinity
    );
    assert_eq!(
        math_types::ieee_float_divide(-infinity, negative_integer_zero),
        -infinity
    );
    for numerator in [1.0, -100.0, f32::MAX, -f32::MAX, 0.0, -0.0] {
        assert_eq!(math_types::ieee_float_divide(numerator, infinity), 0.0);
        assert_eq!(math_types::ieee_float_divide(numerator, -infinity), 0.0);
    }
    for (left, right) in [(0.0f32, 0.0f32), (0.0, -0.0), (-0.0, 0.0), (-0.0, -0.0)] {
        assert!(math_types::ieee_float_divide(left, right).is_nan());
    }
    for (left, right) in [
        (infinity, infinity),
        (infinity, -infinity),
        (-infinity, infinity),
        (-infinity, -infinity),
    ] {
        assert!(math_types::ieee_float_divide(left, right).is_nan());
    }
    assert!(math_types::ieee_float_divide(nan, 1.0).is_nan());
    assert!(math_types::ieee_float_divide(infinity, nan).is_nan());
}

#[test]
fn bit_cast() {
    // SAFETY: u32 and f32 have equal size, and every bit pattern is a valid f32.
    unsafe {
        assert_eq!(math_types::bit_cast::<f32, u32>(0x3f80_0000), 1.0);
        assert_eq!(
            math_types::bit_cast::<f32, u32>((1 << 31) | 0x3f80_0000),
            -1.0
        );
        assert_eq!(math_types::bit_cast::<f32, u32>(0x7f80_0000), f32::INFINITY);
        assert_eq!(
            math_types::bit_cast::<f32, u32>((1 << 31) | 0x7f80_0000),
            f32::NEG_INFINITY
        );
        assert!(math_types::bit_cast::<f32, u32>(0x7fc0_0000).is_nan());
    }
}

#[test]
fn mixed_integer_cmp_equal() {
    assert!(cmp_equal(0i32, 0u32));
    assert!(!cmp_equal(1i32, 0u32));
    assert!(cmp_equal(-100i32, -100i8));
    assert!(cmp_equal(100i8, 100u32));
    assert!(!cmp_equal(-1i8, 0xffu8));
    assert!(!cmp_equal(0xffu8, -1i8));
    assert!(!cmp_equal(-1i16, 0xffffu16));
    assert!(!cmp_equal(0xffffu16, -1i16));
    assert!(!cmp_equal(-1i32, 0xffff_ffffu32));
    assert!(!cmp_equal(0xffff_ffffu32, -1i32));
    assert!(!cmp_equal(-1i64, u64::MAX));
    assert!(!cmp_equal(u64::MAX, -1i64));
}

#[test]
fn mixed_integer_cmp_not_equal() {
    assert!(!cmp_not_equal(0i32, 0u32));
    assert!(cmp_not_equal(1i32, 0u32));
    assert!(!cmp_not_equal(-100i32, -100i8));
    assert!(!cmp_not_equal(100i8, 100u32));
    for result in [
        cmp_not_equal(-1i8, 0xffu8),
        cmp_not_equal(0xffu8, -1i8),
        cmp_not_equal(-1i16, 0xffffu16),
        cmp_not_equal(0xffffu16, -1i16),
        cmp_not_equal(-1i32, 0xffff_ffffu32),
        cmp_not_equal(0xffff_ffffu32, -1i32),
        cmp_not_equal(-1i64, u64::MAX),
        cmp_not_equal(u64::MAX, -1i64),
    ] {
        assert!(result);
    }
}

#[test]
fn mixed_integer_cmp_less() {
    assert!(cmp_less(0i32, 1i32));
    assert!(!cmp_less(0i32, 0i32));
    assert!(!cmp_less(1i32, 0i32));
    assert!(cmp_less(0u32, 1i32));
    assert!(!cmp_less(0u32, 0i32));
    assert!(!cmp_less(1u32, 0i32));
    assert!(cmp_less(0i32, 1u32));
    assert!(!cmp_less(0i32, 0u32));
    assert!(!cmp_less(1i32, 0u32));
    assert!(!cmp_less(0xffu8, -1i8));
    assert!(cmp_less(-1i8, 0xffu8));
    assert!(!cmp_less(2i32, 2u32));
    assert!(!cmp_less(2u32, 2i32));
    assert!(cmp_less(-128i32, 3u32));
    assert!(!cmp_less(3u32, -128i32));
}

#[test]
fn mixed_integer_cmp_greater() {
    assert!(cmp_greater(1i32, 0i32));
    assert!(!cmp_greater(0i32, 0i32));
    assert!(!cmp_greater(0i32, 1i32));
    assert!(cmp_greater(1i32, 0u32));
    assert!(!cmp_greater(0i32, 0u32));
    assert!(!cmp_greater(0i32, 1u32));
    assert!(cmp_greater(1u32, 0i32));
    assert!(!cmp_greater(0u32, 0i32));
    assert!(!cmp_greater(0u32, 1i32));
    assert!(cmp_greater(0xffu8, -1i8));
    assert!(!cmp_greater(-1i8, 0xffu8));
    assert!(!cmp_greater(2i32, 2u32));
    assert!(!cmp_greater(2u32, 2i32));
    assert!(cmp_greater(3u32, -128i32));
    assert!(!cmp_greater(-128i32, 3u32));
}

#[test]
fn mixed_integer_cmp_less_equal() {
    assert!(cmp_less_equal(0i32, 1i32));
    assert!(cmp_less_equal(0i32, 0i32));
    assert!(!cmp_less_equal(1i32, 0i32));
    assert!(cmp_less_equal(0u32, 1i32));
    assert!(cmp_less_equal(0u32, 0i32));
    assert!(!cmp_less_equal(1u32, 0i32));
    assert!(cmp_less_equal(0i32, 1u32));
    assert!(cmp_less_equal(0i32, 0u32));
    assert!(!cmp_less_equal(1i32, 0u32));
    assert!(!cmp_less_equal(0xffu8, -1i8));
    assert!(cmp_less_equal(-1i8, 0xffu8));
    assert!(cmp_less_equal(2i32, 2u32));
    assert!(cmp_less_equal(2u32, 2i32));
    assert!(cmp_less_equal(-128i32, 3u32));
    assert!(!cmp_less_equal(3u32, -128i32));
}

#[test]
fn mixed_integer_cmp_greater_equal() {
    assert!(cmp_greater_equal(1i32, 0i32));
    assert!(cmp_greater_equal(0i32, 0i32));
    assert!(!cmp_greater_equal(0i32, 1i32));
    assert!(cmp_greater_equal(1i32, 0u32));
    assert!(cmp_greater_equal(0i32, 0u32));
    assert!(!cmp_greater_equal(0i32, 1u32));
    assert!(cmp_greater_equal(1u32, 0i32));
    assert!(cmp_greater_equal(0u32, 0i32));
    assert!(!cmp_greater_equal(0u32, 1i32));
    assert!(!cmp_greater_equal(-1i8, 0xffu8));
    assert!(cmp_greater_equal(0xffu8, -1i8));
    assert!(cmp_greater_equal(2u32, 2i32));
    assert!(cmp_greater_equal(2i32, 2u32));
    assert!(cmp_greater_equal(3u32, -128i32));
    assert!(!cmp_greater_equal(-128i32, 3u32));
}

#[test]
fn clamp_cast() {
    assert_eq!(math_types::clamp_cast::<i32, _>(0u32), 0);
    assert_eq!(math_types::clamp_cast::<i8, _>(0xffu32), 0x7f);
    assert_eq!(math_types::clamp_cast::<i16, _>(0xffffu32), 0x7fff);
    assert_eq!(math_types::clamp_cast::<i32, _>(u32::MAX), i32::MAX);
    assert_eq!(math_types::clamp_cast::<i64, _>(u64::MAX), i64::MAX);
    assert_eq!(math_types::clamp_cast::<u8, _>(-1i32), 0);
    assert_eq!(math_types::clamp_cast::<u8, _>(256i32), 255);
    assert_eq!(math_types::clamp_cast::<u8, _>(256u32), 255);
}

#[test]
fn count_leading_zeros() {
    assert_eq!(bitwise::clz32(1u32), 31);
    assert_eq!(bitwise::clz32(u32::MAX), 0);
    for bit in 0..32 {
        assert_eq!(bitwise::clz32(1u32 << bit), 31 - bit);
        assert_eq!(
            bitwise::clz32((1u32 << bit) | (1u32 << bit).wrapping_sub(1)),
            31 - bit
        );
    }
    assert_eq!(bitwise::clz64(1u64), 63);
    assert_eq!(bitwise::clz64(u64::MAX), 0);
    for bit in 0..64 {
        assert_eq!(bitwise::clz64(1u64 << bit), 63 - bit);
        assert_eq!(
            bitwise::clz64((1u64 << bit) | (1u64 << bit).wrapping_sub(1)),
            63 - bit
        );
    }
}

#[test]
fn rotate_left_32() {
    assert_eq!(bitwise::rotate_left32(0xabcd_ef01, 24), 0x01ab_cdef);
    assert_eq!(bitwise::rotate_left32(0xffff_0000, 16), 0x0000_ffff);
}

#[test]
fn most_significant_bit() {
    for (value, expected) in [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 2),
        (4, 3),
        (5, 3),
        (6, 3),
        (7, 3),
        (8, 4),
        (9, 4),
    ] {
        assert_eq!(bitwise::most_significant_bit(value), expected);
    }
    for shift in 0..29 {
        assert_eq!(bitwise::most_significant_bit(10u32 << shift), 4 + shift);
    }
    assert_eq!(bitwise::most_significant_bit(u32::MAX), 32);
}

#[test]
fn round_up_to_multiple_of() {
    assert_eq!(math_types::round_up_to_multiple_of::<4, _>(0), 0);
    assert_eq!(math_types::round_up_to_multiple_of::<4, _>(3), 4);
    assert_eq!(math_types::round_up_to_multiple_of::<8, _>(16), 16);
    assert_eq!(math_types::round_up_to_multiple_of::<8, _>(24), 24);
    assert_eq!(math_types::round_up_to_multiple_of::<8, _>(25), 32);
    assert_eq!(math_types::round_up_to_multiple_of::<8, _>(31), 32);
    assert_eq!(math_types::round_up_to_multiple_of::<8, _>(32), 32);
    for value in 0usize..10 {
        assert_eq!(math_types::round_up_to_multiple_of::<1, _>(value), value);
    }
    assert_eq!(math_types::round_up_to_multiple_of::<2, _>(usize::MAX), 0);
}

#[test]
fn positive_modulo() {
    assert_eq!(positive_mod(1.0, 1.0), 0.0);
    assert_eq!(positive_mod(10.0, 7.0), 3.0);
    assert_eq!(positive_mod(-4.0, 3.0), 2.0);
    assert_eq!(positive_mod(-5.5, 7.0), 1.5);
    assert_eq!(positive_mod(-5.5, -70.0), 64.5);
    assert_eq!(positive_mod(45.5, -12.0), 9.5);
}

#[test]
fn count_set_bits() {
    assert_eq!(bitwise::count_set_bits(0u32), 0);
    assert_eq!(bitwise::count_set_bits(1u32), 1);
    assert_eq!(bitwise::count_set_bits(0x8000_0000u32), 1);
    assert_eq!(bitwise::count_set_bits(0b10010101100110u32), 7);
    assert_eq!(bitwise::count_set_bits(!0b10010101100110u32), 32 - 7);

    assert_eq!(bitwise::count_set_bits(0u64), 0);
    assert_eq!(bitwise::count_set_bits(1u64), 1);
    assert_eq!(bitwise::count_set_bits(0x8000_0000_0000_0000u64), 1);
    assert_eq!(bitwise::count_set_bits(0b10010101100110u64), 7);
    assert_eq!(bitwise::count_set_bits(!0b10010101100110u64), 64 - 7);

    assert_eq!(bitwise::count_set_bits_fallback(u64::from(0u32)), 0);
    assert_eq!(bitwise::count_set_bits_fallback(u64::from(1u32)), 1);
    assert_eq!(
        bitwise::count_set_bits_fallback(u64::from(0x8000_0000u32)),
        1
    );
    assert_eq!(
        bitwise::count_set_bits_fallback(u64::from(0b10010101100110u32)),
        7
    );
    assert_eq!(
        bitwise::count_set_bits_fallback(u64::from(!0b10010101100110u32)),
        32 - 7
    );

    assert_eq!(bitwise::count_set_bits_fallback(0u64), 0);
    assert_eq!(bitwise::count_set_bits_fallback(1u64), 1);
    assert_eq!(
        bitwise::count_set_bits_fallback(0x8000_0000_0000_0000u64),
        1
    );
    assert_eq!(bitwise::count_set_bits_fallback(0b10010101100110u64), 7);
    assert_eq!(
        bitwise::count_set_bits_fallback(!0b10010101100110u64),
        64 - 7
    );
}

#[test]
fn compact_bitmask() {
    for (value, mask, expected) in [
        (0x0000_0000, 0x0000_0000, 0),
        (0xffff_ffff, 0x0000_0000, 0),
        (0x0000_0000, 0x1000_1000, 0),
        (0x0000_1000, 0x1000_1000, 1),
        (0x1000_0000, 0x1000_1000, 2),
        (0x1000_1000, 0x1000_1000, 3),
        (0x1010_1000, 0x1000_1000, 3),
        (0xffff_ffff, 0x1000_1000, 3),
        (0xffff_ffff, 0x1000_1010, 7),
        (0x1000_0000, 0x1000_1010, 4),
    ] {
        assert_eq!(bitwise::compact_bitmask_value(value, mask), expected);
    }
}

#[test]
fn expand_compacted_bitmask() {
    for (value, mask, expected) in [
        (0x0000_0000, 0x0000_0000, 0),
        (0xffff_ffff, 0x0000_0000, 0),
        (0x0000_0000, 0x1000_1000, 0),
        (1, 0x1000_1000, 0x0000_1000),
        (2, 0x1000_1000, 0x1000_0000),
        (3, 0x1000_1000, 0x1000_1000),
        (0xffff_ffff, 0x1000_1000, 0x1000_1000),
        (7, 0x1000_1010, 0x1000_1010),
        (4, 0x1000_1010, 0x1000_0000),
    ] {
        assert_eq!(
            bitwise::expand_compacted_bitmask_value(value, mask),
            expected
        );
    }
}

#[test]
fn iterate_bit_combinations_in_mask() {
    assert_eq!(
        bitwise::iterate_bit_combinations_in_mask(0u32)
            .into_iter()
            .collect::<Vec<_>>(),
        [0]
    );
    assert_eq!(
        bitwise::iterate_bit_combinations_in_mask(1u32)
            .into_iter()
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert_eq!(
        bitwise::iterate_bit_combinations_in_mask(0x1001u32)
            .into_iter()
            .collect::<Vec<_>>(),
        [0x1001, 0x1000, 1, 0]
    );
    assert_eq!(
        bitwise::iterate_bit_combinations_in_mask(0x11u32)
            .into_iter()
            .collect::<Vec<_>>(),
        [0x11, 0x10, 1, 0]
    );
    assert_eq!(
        bitwise::iterate_bit_combinations_in_mask(0x13u32)
            .into_iter()
            .collect::<Vec<_>>(),
        [0x13, 0x12, 0x11, 0x10, 3, 2, 1, 0]
    );
}
