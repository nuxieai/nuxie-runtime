//! One distinct Rust-language adaptation for each pinned
//! `tests/unit_tests/runtime/type_conversions_test.cpp` case.
//!
//! Rust's primitive `checked_mul` and `overflowing_mul` operations are the
//! language-native authority. No test-local multiplication helper is used.

#[test]
fn checked_mul_accepts_zero_inputs() {
    let mut out = 0xdead_beef_usize;
    let result = 0usize.checked_mul(0);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, 0);

    let result = 0usize.checked_mul(usize::MAX);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, 0);

    let result = usize::MAX.checked_mul(0);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, 0);
}

#[test]
fn checked_mul_preserves_identity() {
    let result = 1u32.checked_mul(12_345);
    assert!(result.is_some());
    let mut out32 = result.unwrap();
    assert_eq!(out32, 12_345);
    let result = 67_890u32.checked_mul(1);
    assert!(result.is_some());
    out32 = result.unwrap();
    assert_eq!(out32, 67_890);

    let result = 1u64.checked_mul(u64::MAX);
    assert!(result.is_some());
    let out64 = result.unwrap();
    assert_eq!(out64, u64::MAX);
}

#[test]
fn checked_mul_computes_small_products() {
    let result = 2usize.checked_mul(3);
    assert!(result.is_some());
    let mut out = result.unwrap();
    assert_eq!(out, 6);
    let result = 7usize.checked_mul(11);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, 77);
}

#[test]
fn checked_mul_succeeds_just_below_overflow() {
    {
        let max32 = u32::MAX;
        let result = (max32 / 2).checked_mul(2);
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, (max32 / 2) * 2);
    }
    {
        let max64 = u64::MAX;
        let result = (max64 / 2).checked_mul(2);
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, (max64 / 2) * 2);
    }
}

#[test]
fn checked_mul_detects_overflow_at_boundary() {
    {
        let max32 = u32::MAX;
        let result = (max32 / 2 + 1).checked_mul(2);
        assert!(result.is_none());
    }
    {
        let max64 = u64::MAX;
        let result = (max64 / 2 + 1).checked_mul(2);
        assert!(result.is_none());
    }
}

#[test]
fn checked_mul_detects_square_overflow() {
    let result = u32::MAX.checked_mul(u32::MAX);
    assert!(result.is_none());

    let result = u64::MAX.checked_mul(u64::MAX);
    assert!(result.is_none());
}

#[test]
fn checked_mul_covers_narrow_unsigned_widths() {
    {
        let result = 15u8.checked_mul(17);
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, 255);
        let result = 16u8.checked_mul(16);
        assert!(result.is_none());
    }
    {
        let result = 255u16.checked_mul(257);
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, 65_535);
        let result = 256u16.checked_mul(257);
        assert!(result.is_none());
    }
}

#[test]
fn checked_mul_covers_usize_directly() {
    let max_size = usize::MAX;
    let result = 0usize.checked_mul(max_size);
    assert!(result.is_some());
    let mut out = result.unwrap();
    assert_eq!(out, 0);
    let result = 7usize.checked_mul(11);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, 77);
    let result = (max_size / 2).checked_mul(2);
    assert!(result.is_some());
    out = result.unwrap();
    assert_eq!(out, (max_size / 2) * 2);
    assert!((max_size / 2 + 1).checked_mul(2).is_none());
    assert!(max_size.checked_mul(max_size).is_none());
}

#[test]
fn checked_mul_tolerates_output_aliasing_input() {
    {
        let mut x = 7usize;
        let result = x.checked_mul(11);
        assert!(result.is_some());
        x = result.unwrap();
        assert_eq!(x, 77);
    }
    {
        let mut y = 13usize;
        let result = 3usize.checked_mul(y);
        assert!(result.is_some());
        y = result.unwrap();
        assert_eq!(y, 39);
    }
    {
        let z = usize::MAX;
        assert!(z.checked_mul(2).is_none());
    }
}

#[test]
fn mul_overflows_inverts_checked_mul() {
    let max_size = usize::MAX;
    assert_eq!(
        max_size.overflowing_mul(2).1,
        max_size.checked_mul(2).is_none()
    );
    assert_eq!(
        7usize.overflowing_mul(11).1,
        7usize.checked_mul(11).is_none()
    );
    assert_eq!(
        0usize.overflowing_mul(max_size).1,
        0usize.checked_mul(max_size).is_none()
    );
}

#[test]
fn mul_overflows_captures_decoder_shaped_sizing() {
    assert!(!65_537usize.overflowing_mul(65_537).1);
    assert!(65_537u32.overflowing_mul(65_537).1);
}
