use super::positive_mod;

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
fn wave_c3_math_014_positive_mod() {
    assert_eq!(positive_mod(1.0, 1.0), 0.0);
    assert_eq!(positive_mod(10.0, 7.0), 3.0);
    assert_eq!(positive_mod(-4.0, 3.0), 2.0);
    assert_eq!(positive_mod(-5.5, 7.0), 1.5);
    assert_eq!(positive_mod(-5.5, -70.0), 64.5);
    assert_eq!(positive_mod(45.5, -12.0), 9.5);
}
