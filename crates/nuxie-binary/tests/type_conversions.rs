// Direct safe-Rust ports of the complete pinned
// `tests/unit_tests/runtime/type_conversions_test.cpp` denominator. Rust's
// primitive `checked_mul` is the intentional language-native owner for the
// upstream checkedMul/mulOverflows helpers.

fn mul_overflows<T>(left: T, right: T) -> bool
where
    T: Copy + CheckedMul,
{
    left.checked_mul(right).is_none()
}

trait CheckedMul: Sized {
    fn checked_mul(self, right: Self) -> Option<Self>;
}

macro_rules! checked_mul_impl {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl CheckedMul for $ty {
                fn checked_mul(self, right: Self) -> Option<Self> {
                    <$ty>::checked_mul(self, right)
                }
            }
        )+
    };
}

checked_mul_impl!(u32, usize);

#[test]
fn checked_mul_accepts_zero_inputs() {
    assert_eq!(0usize.checked_mul(0), Some(0));
    assert_eq!(0usize.checked_mul(usize::MAX), Some(0));
    assert_eq!(usize::MAX.checked_mul(0), Some(0));
}

#[test]
fn checked_mul_preserves_identity() {
    assert_eq!(1u32.checked_mul(12_345), Some(12_345));
    assert_eq!(67_890u32.checked_mul(1), Some(67_890));
    assert_eq!(1u64.checked_mul(u64::MAX), Some(u64::MAX));
}

#[test]
fn checked_mul_computes_small_products() {
    assert_eq!(2usize.checked_mul(3), Some(6));
    assert_eq!(7usize.checked_mul(11), Some(77));
}

#[test]
fn checked_mul_succeeds_just_below_overflow() {
    assert_eq!((u32::MAX / 2).checked_mul(2), Some((u32::MAX / 2) * 2));
    assert_eq!((u64::MAX / 2).checked_mul(2), Some((u64::MAX / 2) * 2));
}

#[test]
fn checked_mul_detects_overflow_at_boundary() {
    assert_eq!((u32::MAX / 2 + 1).checked_mul(2), None);
    assert_eq!((u64::MAX / 2 + 1).checked_mul(2), None);
}

#[test]
fn checked_mul_detects_square_overflow() {
    assert_eq!(u32::MAX.checked_mul(u32::MAX), None);
    assert_eq!(u64::MAX.checked_mul(u64::MAX), None);
}

#[test]
fn checked_mul_covers_narrow_unsigned_widths() {
    assert_eq!(15u8.checked_mul(17), Some(255));
    assert_eq!(16u8.checked_mul(16), None);
    assert_eq!(255u16.checked_mul(257), Some(65_535));
    assert_eq!(256u16.checked_mul(257), None);
}

#[test]
fn checked_mul_covers_usize_directly() {
    assert_eq!(0usize.checked_mul(usize::MAX), Some(0));
    assert_eq!(7usize.checked_mul(11), Some(77));
    assert_eq!((usize::MAX / 2).checked_mul(2), Some((usize::MAX / 2) * 2));
    assert_eq!((usize::MAX / 2 + 1).checked_mul(2), None);
    assert_eq!(usize::MAX.checked_mul(usize::MAX), None);
}

#[test]
fn checked_mul_tolerates_output_aliasing_input() {
    let mut x = 7usize;
    x = x.checked_mul(11).expect("7 * 11");
    assert_eq!(x, 77);

    let mut y = 13usize;
    y = 3usize.checked_mul(y).expect("3 * 13");
    assert_eq!(y, 39);

    let z = usize::MAX;
    assert_eq!(z.checked_mul(2), None);
}

#[test]
fn mul_overflows_inverts_checked_mul() {
    assert_eq!(
        mul_overflows(usize::MAX, 2),
        usize::MAX.checked_mul(2).is_none()
    );
    assert_eq!(mul_overflows(7usize, 11), 7usize.checked_mul(11).is_none());
    assert_eq!(
        mul_overflows(0usize, usize::MAX),
        0usize.checked_mul(usize::MAX).is_none()
    );
}

#[test]
fn mul_overflows_captures_decoder_shaped_sizing() {
    assert!(!mul_overflows(65_537usize, 65_537));
    assert!(mul_overflows(65_537u32, 65_537));
}
