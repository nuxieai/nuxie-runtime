use super::positive_mod;

#[derive(Clone, Copy)]
enum MixedInteger {
    Signed(i128),
    Unsigned(u128),
}

macro_rules! signed_mixed_integers {
    ($($kind:ty),+ $(,)?) => {$(
        impl From<$kind> for MixedInteger {
            fn from(value: $kind) -> Self {
                Self::Signed(value as i128)
            }
        }
    )+};
}

macro_rules! unsigned_mixed_integers {
    ($($kind:ty),+ $(,)?) => {$(
        impl From<$kind> for MixedInteger {
            fn from(value: $kind) -> Self {
                Self::Unsigned(value as u128)
            }
        }
    )+};
}

signed_mixed_integers!(i8, i16, i32, i64);
unsigned_mixed_integers!(u8, u16, u32, u64);

// Rust deliberately rejects C++'s mixed-sign primitive comparisons. These
// four helpers are the explicit cxx-language-only adaptation of the pinned
// `cmp_*` templates, using a lossless common representation rather than Rust
// casts that would reproduce C++'s unsafe default promotions.
fn mixed_less(left: impl Into<MixedInteger>, right: impl Into<MixedInteger>) -> bool {
    match (left.into(), right.into()) {
        (MixedInteger::Signed(left), MixedInteger::Signed(right)) => left < right,
        (MixedInteger::Unsigned(left), MixedInteger::Unsigned(right)) => left < right,
        (MixedInteger::Signed(left), MixedInteger::Unsigned(right)) => {
            left < 0 || (left as u128) < right
        }
        (MixedInteger::Unsigned(left), MixedInteger::Signed(right)) => {
            right >= 0 && left < right as u128
        }
    }
}

fn mixed_equal(
    left: impl Into<MixedInteger> + Copy,
    right: impl Into<MixedInteger> + Copy,
) -> bool {
    !mixed_less(left, right) && !mixed_less(right, left)
}

fn mixed_greater(
    left: impl Into<MixedInteger> + Copy,
    right: impl Into<MixedInteger> + Copy,
) -> bool {
    mixed_less(right, left)
}

fn mixed_not_equal(
    left: impl Into<MixedInteger> + Copy,
    right: impl Into<MixedInteger> + Copy,
) -> bool {
    !mixed_equal(left, right)
}

fn mixed_less_equal(
    left: impl Into<MixedInteger> + Copy,
    right: impl Into<MixedInteger> + Copy,
) -> bool {
    !mixed_greater(left, right)
}

fn mixed_greater_equal(
    left: impl Into<MixedInteger> + Copy,
    right: impl Into<MixedInteger> + Copy,
) -> bool {
    !mixed_less(left, right)
}

#[test]
fn wave_c3_math_001_ieee_float_divide() {
    let infinity = f32::INFINITY;
    let nan = f32::NAN;
    // Pinned C++ spells this `-0`: integer negation occurs before conversion,
    // so the actual operand is positive IEEE zero.
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

    assert_eq!(1.0f32 / infinity, 0.0);
    assert_eq!(-100.0f32 / infinity, 0.0);
    assert_eq!(f32::MAX / infinity, 0.0);
    assert_eq!(f32::MAX / -infinity, 0.0);
    assert_eq!(-f32::MAX / -infinity, 0.0);
    assert_eq!(-f32::MAX / infinity, 0.0);
    assert_eq!(0.0f32 / infinity, 0.0);
    assert_eq!(0.0f32 / -infinity, 0.0);
    assert_eq!(negative_integer_zero / -infinity, 0.0);
    assert_eq!(negative_integer_zero / infinity, 0.0);

    assert!((0.0f32 / 0.0).is_nan());
    assert!((0.0f32 / negative_integer_zero).is_nan());
    assert!((negative_integer_zero / 0.0).is_nan());
    assert!((negative_integer_zero / negative_integer_zero).is_nan());

    assert!((infinity / infinity).is_nan());
    assert!((infinity / -infinity).is_nan());
    assert!((-infinity / infinity).is_nan());
    assert!((infinity / -infinity).is_nan());

    assert!((nan / 1.0).is_nan());
    assert!((infinity / nan).is_nan());
}

#[test]
fn wave_c3_math_002_bit_cast() {
    assert_eq!(f32::from_bits(0x3f80_0000), 1.0);
    assert_eq!(f32::from_bits((1u32 << 31) | 0x3f80_0000), -1.0);
    assert_eq!(f32::from_bits(0x7f80_0000), f32::INFINITY);
    assert_eq!(
        f32::from_bits((1u32 << 31) | 0x7f80_0000),
        f32::NEG_INFINITY
    );
    assert!(f32::from_bits(0x7fc0_0000).is_nan());
}

#[test]
fn wave_c3_math_003_cmp_equal() {
    assert!(mixed_equal(0i32, 0u32));
    assert!(!mixed_equal(1i32, 0u32));
    assert!(mixed_equal(-100i32, -100i8));
    assert!(mixed_equal(100i8, 100u32));
    assert!(!mixed_equal(-1i8, 0xffu8));
    assert!(!mixed_equal(0xffu8, -1i8));
    assert!(!mixed_equal(-1i16, 0xffffu16));
    assert!(!mixed_equal(0xffffu16, -1i16));
    assert!(!mixed_equal(-1i32, 0xffff_ffffu32));
    assert!(!mixed_equal(0xffff_ffffu32, -1i32));
    assert!(!mixed_equal(-1i64, u64::MAX));
    assert!(!mixed_equal(u64::MAX, -1i64));
}

#[test]
fn wave_c3_math_004_cmp_not_equal() {
    assert!(!mixed_not_equal(0i32, 0u32));
    assert!(mixed_not_equal(1i32, 0u32));
    assert!(!mixed_not_equal(-100i32, -100i8));
    assert!(!mixed_not_equal(100i8, 100u32));
    assert!(mixed_not_equal(-1i8, 0xffu8));
    assert!(mixed_not_equal(0xffu8, -1i8));
    assert!(mixed_not_equal(-1i16, 0xffffu16));
    assert!(mixed_not_equal(0xffffu16, -1i16));
    assert!(mixed_not_equal(-1i32, 0xffff_ffffu32));
    assert!(mixed_not_equal(0xffff_ffffu32, -1i32));
    assert!(mixed_not_equal(-1i64, u64::MAX));
    assert!(mixed_not_equal(u64::MAX, -1i64));
}

#[test]
fn wave_c3_math_005_cmp_less() {
    assert!(mixed_less(0i32, 1i32));
    assert!(!mixed_less(0i32, 0i32));
    assert!(!mixed_less(1i32, 0i32));
    assert!(mixed_less(0u32, 1i32));
    assert!(!mixed_less(0u32, 0i32));
    assert!(!mixed_less(1u32, 0i32));
    assert!(mixed_less(0i32, 1u32));
    assert!(!mixed_less(0i32, 0u32));
    assert!(!mixed_less(1i32, 0u32));
    assert!(!mixed_less(0xffu8, -1i8));
    assert!(mixed_less(-1i8, 0xffu8));
    assert!(!mixed_less(2i32, 2u32));
    assert!(!mixed_less(2u32, 2i32));
    assert!(mixed_less(-128i32, 3u32));
    assert!(!mixed_less(3u32, -128i32));
}

#[test]
fn wave_c3_math_006_cmp_greater() {
    assert!(mixed_greater(1i32, 0i32));
    assert!(!mixed_greater(0i32, 0i32));
    assert!(!mixed_greater(0i32, 1i32));
    assert!(mixed_greater(1i32, 0u32));
    assert!(!mixed_greater(0i32, 0u32));
    assert!(!mixed_greater(0i32, 1u32));
    assert!(mixed_greater(1u32, 0i32));
    assert!(!mixed_greater(0u32, 0i32));
    assert!(!mixed_greater(0u32, 1i32));
    assert!(mixed_greater(0xffu8, -1i8));
    assert!(!mixed_greater(-1i8, 0xffu8));
    assert!(!mixed_greater(2i32, 2u32));
    assert!(!mixed_greater(2u32, 2i32));
    assert!(mixed_greater(3u32, -128i32));
    assert!(!mixed_greater(-128i32, 3u32));
}

#[test]
fn wave_c3_math_007_cmp_less_equal() {
    assert!(mixed_less_equal(0i32, 1i32));
    assert!(mixed_less_equal(0i32, 0i32));
    assert!(!mixed_less_equal(1i32, 0i32));
    assert!(mixed_less_equal(0u32, 1i32));
    assert!(mixed_less_equal(0u32, 0i32));
    assert!(!mixed_less_equal(1u32, 0i32));
    assert!(mixed_less_equal(0i32, 1u32));
    assert!(mixed_less_equal(0i32, 0u32));
    assert!(!mixed_less_equal(1i32, 0u32));
    assert!(!mixed_less_equal(0xffu8, -1i8));
    assert!(mixed_less_equal(-1i8, 0xffu8));
    assert!(mixed_less_equal(2i32, 2u32));
    assert!(mixed_less_equal(2u32, 2i32));
    assert!(mixed_less_equal(-128i32, 3u32));
    assert!(!mixed_less_equal(3u32, -128i32));
}

#[test]
fn wave_c3_math_008_cmp_greater_equal() {
    assert!(mixed_greater_equal(1i32, 0i32));
    assert!(mixed_greater_equal(0i32, 0i32));
    assert!(!mixed_greater_equal(0i32, 1i32));
    assert!(mixed_greater_equal(1i32, 0u32));
    assert!(mixed_greater_equal(0i32, 0u32));
    assert!(!mixed_greater_equal(0i32, 1u32));
    assert!(mixed_greater_equal(1u32, 0i32));
    assert!(mixed_greater_equal(0u32, 0i32));
    assert!(!mixed_greater_equal(0u32, 1i32));
    assert!(!mixed_greater_equal(-1i8, 0xffu8));
    assert!(mixed_greater_equal(0xffu8, -1i8));
    assert!(mixed_greater_equal(2u32, 2i32));
    assert!(mixed_greater_equal(2i32, 2u32));
    assert!(mixed_greater_equal(3u32, -128i32));
    assert!(!mixed_greater_equal(-128i32, 3u32));
}

#[test]
fn wave_c3_math_009_clamp_cast() {
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
fn wave_c3_math_010_clz() {
    assert_eq!(1u32.leading_zeros(), 31);
    assert_eq!(u32::MAX.leading_zeros(), 0);
    for bit in 0..32 {
        let leading = 1u32 << bit;
        let random_low = (unsafe { libc::rand() } as u32) & leading.wrapping_sub(1);
        assert_eq!(leading.leading_zeros(), 31 - bit);
        assert_eq!((leading | random_low).leading_zeros(), 31 - bit);
    }

    assert_eq!(1u64.leading_zeros(), 63);
    assert_eq!(u64::MAX.leading_zeros(), 0);
    for bit in 0..64 {
        let leading = 1u64 << bit;
        let random_low = (unsafe { libc::rand() } as u64) & leading.wrapping_sub(1);
        assert_eq!(leading.leading_zeros(), 63 - bit);
        assert_eq!((leading | random_low).leading_zeros(), 63 - bit);
    }
}

#[test]
fn wave_c3_math_011_rotateleft32() {
    assert_eq!(0xabcd_ef01u32.rotate_left(24), 0x01ab_cdef);
    assert_eq!(0xffff_0000u32.rotate_left(16), 0x0000_ffff);
}

#[test]
fn wave_c3_math_012_msb() {
    let msb = |value: u32| u32::BITS - value.leading_zeros();
    assert_eq!(msb(0), 0);
    assert_eq!(msb(1), 1);
    assert_eq!(msb(2), 2);
    assert_eq!(msb(3), 2);
    assert_eq!(msb(4), 3);
    assert_eq!(msb(5), 3);
    assert_eq!(msb(6), 3);
    assert_eq!(msb(7), 3);
    assert_eq!(msb(8), 4);
    assert_eq!(msb(9), 4);
    for bit in 0..29 {
        assert_eq!(msb(10u32 << bit), 4 + bit);
    }
    assert_eq!(msb(u32::MAX), 32);
}

#[test]
fn wave_c3_math_013_round_up_to_multiple_of() {
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
fn wave_c3_math_014_positive_mod() {
    assert_eq!(positive_mod(1.0, 1.0), 0.0);
    assert_eq!(positive_mod(10.0, 7.0), 3.0);
    assert_eq!(positive_mod(-4.0, 3.0), 2.0);
    assert_eq!(positive_mod(-5.5, 7.0), 1.5);
    assert_eq!(positive_mod(-5.5, -70.0), 64.5);
    assert_eq!(positive_mod(45.5, -12.0), 9.5);
}

#[test]
fn wave_c3_math_015_count_set_bits() {
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

    // Rust's primitive is also the fallback owner. Repeat the pinned fallback
    // assertion sequence rather than creating a second test-local algorithm.
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
