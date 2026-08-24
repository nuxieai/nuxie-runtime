//! Rust-native ports of pinned `tests/unit_tests/runtime/math_test.cpp`.
//!
//! The upstream file exercises C++ utility templates. Rust's primitive integer
//! and float operations are the corresponding owner; this test deliberately
//! does not add production wrappers merely to reproduce C++ template names.

#[derive(Clone, Copy)]
enum IntValue {
    Signed(i128),
    Unsigned(u128),
}

macro_rules! signed_values {
    ($($type:ty),+ $(,)?) => {$(
        impl From<$type> for IntValue {
            fn from(value: $type) -> Self { Self::Signed(value as i128) }
        }
    )+};
}

macro_rules! unsigned_values {
    ($($type:ty),+ $(,)?) => {$(
        impl From<$type> for IntValue {
            fn from(value: $type) -> Self { Self::Unsigned(value as u128) }
        }
    )+};
}

signed_values!(i8, i16, i32, i64, i128, isize);
unsigned_values!(u8, u16, u32, u64, u128, usize);

fn cmp_less(left: impl Into<IntValue>, right: impl Into<IntValue>) -> bool {
    match (left.into(), right.into()) {
        (IntValue::Signed(left), IntValue::Signed(right)) => left < right,
        (IntValue::Unsigned(left), IntValue::Unsigned(right)) => left < right,
        (IntValue::Signed(left), IntValue::Unsigned(right)) => left < 0 || (left as u128) < right,
        (IntValue::Unsigned(left), IntValue::Signed(right)) => right >= 0 && left < right as u128,
    }
}

fn cmp_equal(left: impl Into<IntValue> + Copy, right: impl Into<IntValue> + Copy) -> bool {
    !cmp_less(left, right) && !cmp_less(right, left)
}

fn positive_mod(value: f32, range: f32) -> f32 {
    let range = range.abs();
    ((value % range) + range) % range
}

fn compact_bitmask_value(value: u32, mask: u32) -> u32 {
    let mut compact = 0;
    let mut compact_bit = 1;
    for bit in 0..u32::BITS {
        let source_bit = 1u32 << bit;
        if mask & source_bit != 0 {
            if value & source_bit != 0 {
                compact |= compact_bit;
            }
            compact_bit <<= 1;
        }
    }
    compact
}

fn expand_compacted_bitmask_value(value: u32, mask: u32) -> u32 {
    let mut expanded = 0;
    let mut compact_bit = 1;
    for bit in 0..u32::BITS {
        let destination_bit = 1u32 << bit;
        if mask & destination_bit != 0 {
            if value & compact_bit != 0 {
                expanded |= destination_bit;
            }
            compact_bit <<= 1;
        }
    }
    expanded
}

fn bit_combinations(mask: u32) -> Vec<u32> {
    let mut values = Vec::new();
    let mut value = mask;
    loop {
        values.push(value);
        if value == 0 {
            break;
        }
        value = value.wrapping_sub(1) & mask;
    }
    values
}

#[test]
fn ieee_float_divide() {
    let infinity = f32::INFINITY;
    let nan = f32::NAN;
    // Upstream spells this `-0`; unary minus is applied to integer zero before
    // conversion to float, so it is positive IEEE zero rather than `-0.0f`.
    let negative_integer_zero = (-0i32) as f32;
    assert_eq!(100.0f32 / 10.0, 10.0);
    assert_eq!(5.0f32 / 0.0, infinity);
    assert_eq!(5.0f32 / negative_integer_zero, infinity);
    assert_eq!(-3.0f32 / 0.0, -infinity);
    assert_eq!(-3.0f32 / negative_integer_zero, -infinity);
    assert_eq!(infinity / 0.0, infinity);
    assert_eq!(-infinity / 0.0, -infinity);
    assert_eq!(infinity / negative_integer_zero, infinity);
    assert_eq!(-infinity / negative_integer_zero, -infinity);
    for numerator in [1.0, -100.0, f32::MAX, -f32::MAX, 0.0, -0.0] {
        assert_eq!(numerator / infinity, 0.0);
        assert_eq!(numerator / -infinity, 0.0);
    }
    for (left, right) in [(0.0f32, 0.0f32), (0.0, -0.0), (-0.0, 0.0), (-0.0, -0.0)] {
        assert!((left / right).is_nan());
    }
    for (left, right) in [
        (infinity, infinity),
        (infinity, -infinity),
        (-infinity, infinity),
        (-infinity, -infinity),
    ] {
        assert!((left / right).is_nan());
    }
    assert!((nan / 1.0).is_nan());
    assert!((infinity / nan).is_nan());
}

#[test]
fn bit_cast() {
    assert_eq!(f32::from_bits(0x3f80_0000), 1.0);
    assert_eq!(f32::from_bits((1 << 31) | 0x3f80_0000), -1.0);
    assert_eq!(f32::from_bits(0x7f80_0000), f32::INFINITY);
    assert_eq!(f32::from_bits((1 << 31) | 0x7f80_0000), f32::NEG_INFINITY);
    assert!(f32::from_bits(0x7fc0_0000).is_nan());
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
    assert!(!(!cmp_equal(0i32, 0u32)));
    assert!(!cmp_equal(1i32, 0u32));
    assert!(!(!cmp_equal(-100i32, -100i8)));
    assert!(!(!cmp_equal(100i8, 100u32)));
    for result in [
        !cmp_equal(-1i8, 0xffu8),
        !cmp_equal(0xffu8, -1i8),
        !cmp_equal(-1i16, 0xffffu16),
        !cmp_equal(0xffffu16, -1i16),
        !cmp_equal(-1i32, 0xffff_ffffu32),
        !cmp_equal(0xffff_ffffu32, -1i32),
        !cmp_equal(-1i64, u64::MAX),
        !cmp_equal(u64::MAX, -1i64),
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
    let greater = |left: IntValue, right: IntValue| cmp_less(right, left);
    assert!(greater(1i32.into(), 0i32.into()));
    assert!(!greater(0i32.into(), 0i32.into()));
    assert!(!greater(0i32.into(), 1i32.into()));
    assert!(cmp_less(0u32, 1i32));
    assert!(!cmp_less(0u32, 0i32));
    assert!(!cmp_less(1u32, 0i32));
    assert!(cmp_less(-1i8, 0xffu8));
    assert!(!cmp_less(0xffu8, -1i8));
    assert!(!cmp_less(2u32, 2i32));
    assert!(!cmp_less(2i32, 2u32));
    assert!(cmp_less(-128i32, 3u32));
    assert!(!cmp_less(3u32, -128i32));
}

#[test]
fn mixed_integer_cmp_less_equal() {
    let less_equal = |left: IntValue, right: IntValue| !cmp_less(right, left);
    assert!(less_equal(0i32.into(), 1i32.into()));
    assert!(less_equal(0i32.into(), 0i32.into()));
    assert!(!less_equal(1i32.into(), 0i32.into()));
    assert!(less_equal(0u32.into(), 1i32.into()));
    assert!(less_equal(0u32.into(), 0i32.into()));
    assert!(!less_equal(1u32.into(), 0i32.into()));
    assert!(less_equal(0i32.into(), 1u32.into()));
    assert!(less_equal(0i32.into(), 0u32.into()));
    assert!(!less_equal(1i32.into(), 0u32.into()));
    assert!(!less_equal(0xffu8.into(), (-1i8).into()));
    assert!(less_equal((-1i8).into(), 0xffu8.into()));
    assert!(less_equal(2i32.into(), 2u32.into()));
    assert!(less_equal(2u32.into(), 2i32.into()));
    assert!(less_equal((-128i32).into(), 3u32.into()));
    assert!(!less_equal(3u32.into(), (-128i32).into()));
}

#[test]
fn mixed_integer_cmp_greater_equal() {
    let greater_equal = |left: IntValue, right: IntValue| !cmp_less(left, right);
    assert!(greater_equal(1i32.into(), 0i32.into()));
    assert!(greater_equal(0i32.into(), 0i32.into()));
    assert!(!greater_equal(0i32.into(), 1i32.into()));
    assert!(greater_equal(1i32.into(), 0u32.into()));
    assert!(greater_equal(0i32.into(), 0u32.into()));
    assert!(!greater_equal(0i32.into(), 1u32.into()));
    assert!(greater_equal(1u32.into(), 0i32.into()));
    assert!(greater_equal(0u32.into(), 0i32.into()));
    assert!(!greater_equal(0u32.into(), 1i32.into()));
    assert!(!greater_equal((-1i8).into(), 0xffu8.into()));
    assert!(greater_equal(0xffu8.into(), (-1i8).into()));
    assert!(greater_equal(2u32.into(), 2i32.into()));
    assert!(greater_equal(2i32.into(), 2u32.into()));
    assert!(greater_equal(3u32.into(), (-128i32).into()));
    assert!(!greater_equal((-128i32).into(), 3u32.into()));
}

#[test]
fn clamp_cast() {
    assert_eq!(0u32.min(i32::MAX as u32) as i32, 0);
    assert_eq!(0xffu32.min(i8::MAX as u32) as i8, 0x7f);
    assert_eq!(0xffffu32.min(i16::MAX as u32) as i16, 0x7fff);
    assert_eq!(u32::MAX.min(i32::MAX as u32) as i32, i32::MAX);
    assert_eq!(u64::MAX.min(i64::MAX as u64) as i64, i64::MAX);
    assert_eq!((-1i32).clamp(0, u8::MAX as i32) as u8, 0);
    assert_eq!(256i32.clamp(0, u8::MAX as i32) as u8, 255);
    assert_eq!(256u32.min(u8::MAX as u32) as u8, 255);
}

#[test]
fn count_leading_zeros() {
    assert_eq!(1u32.leading_zeros(), 31);
    assert_eq!(u32::MAX.leading_zeros(), 0);
    for bit in 0..32 {
        assert_eq!((1u32 << bit).leading_zeros(), 31 - bit);
        assert_eq!(
            ((1u32 << bit) | (1u32 << bit).wrapping_sub(1)).leading_zeros(),
            31 - bit
        );
    }
    assert_eq!(1u64.leading_zeros(), 63);
    assert_eq!(u64::MAX.leading_zeros(), 0);
    for bit in 0..64 {
        assert_eq!((1u64 << bit).leading_zeros(), 63 - bit);
        assert_eq!(
            ((1u64 << bit) | (1u64 << bit).wrapping_sub(1)).leading_zeros(),
            63 - bit
        );
    }
}

#[test]
fn rotate_left_32() {
    assert_eq!(0xabcd_ef01u32.rotate_left(24), 0x01ab_cdef);
    assert_eq!(0xffff_0000u32.rotate_left(16), 0x0000_ffff);
}

#[test]
fn most_significant_bit() {
    let msb = |value: u32| u32::BITS - value.leading_zeros();
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
        assert_eq!(msb(value), expected);
    }
    for shift in 0..29 {
        assert_eq!(msb(10u32 << shift), 4 + shift);
    }
    assert_eq!(msb(u32::MAX), 32);
}

#[test]
fn round_up_to_multiple_of() {
    let round =
        |value: usize, multiple: usize| value.wrapping_add(multiple - 1) / multiple * multiple;
    assert_eq!(round(0, 4), 0);
    assert_eq!(round(3, 4), 4);
    assert_eq!(round(16, 8), 16);
    assert_eq!(round(24, 8), 24);
    assert_eq!(round(25, 8), 32);
    assert_eq!(round(31, 8), 32);
    assert_eq!(round(32, 8), 32);
    for value in 0..10 {
        assert_eq!(round(value, 1), value);
    }
    assert_eq!(round(usize::MAX, 2), 0);
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
    assert_eq!(0u32.count_ones(), 0);
    assert_eq!(1u32.count_ones(), 1);
    assert_eq!(0x8000_0000u32.count_ones(), 1);
    assert_eq!(0b10010101100110u32.count_ones(), 7);
    assert_eq!((!0b10010101100110u32).count_ones(), 32 - 7);

    assert_eq!(0u64.count_ones(), 0);
    assert_eq!(1u64.count_ones(), 1);
    assert_eq!(0x8000_0000_0000_0000u64.count_ones(), 1);
    assert_eq!(0b10010101100110u64.count_ones(), 7);
    assert_eq!((!0b10010101100110u64).count_ones(), 64 - 7);

    // Rust's primitive is also the fallback owner, so repeat the upstream
    // fallback assertion sequence instead of inventing a second algorithm.
    assert_eq!(0u32.count_ones(), 0);
    assert_eq!(1u32.count_ones(), 1);
    assert_eq!(0x8000_0000u32.count_ones(), 1);
    assert_eq!(0b10010101100110u32.count_ones(), 7);
    assert_eq!((!0b10010101100110u32).count_ones(), 32 - 7);

    assert_eq!(0u64.count_ones(), 0);
    assert_eq!(1u64.count_ones(), 1);
    assert_eq!(0x8000_0000_0000_0000u64.count_ones(), 1);
    assert_eq!(0b10010101100110u64.count_ones(), 7);
    assert_eq!((!0b10010101100110u64).count_ones(), 64 - 7);
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
        assert_eq!(compact_bitmask_value(value, mask), expected);
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
        assert_eq!(expand_compacted_bitmask_value(value, mask), expected);
    }
}

#[test]
fn iterate_bit_combinations_in_mask() {
    assert_eq!(bit_combinations(0), [0]);
    assert_eq!(bit_combinations(1), [1, 0]);
    assert_eq!(bit_combinations(0x1001), [0x1001, 0x1000, 1, 0]);
    assert_eq!(bit_combinations(0x11), [0x11, 0x10, 1, 0]);
    assert_eq!(bit_combinations(0x13), [0x13, 0x12, 0x11, 0x10, 3, 2, 1, 0]);
}
