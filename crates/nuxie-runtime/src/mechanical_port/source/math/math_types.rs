use core::mem::{MaybeUninit, size_of};
use core::ops::{Add, BitAnd, BitOr, Mul, Not, Sub};

pub const PI: f32 = 3.141_592_7;
pub const SQRT_2: f32 = 1.414_213_5;
pub const EPSILON: f32 = 1.0 / 4096.0;

pub fn nearly_zero(value: f32, tolerance: f32) -> bool {
    assert!(tolerance >= 0.0);
    value.abs() <= tolerance
}
pub fn nearly_equal(a: f32, b: f32, tolerance: f32) -> bool {
    nearly_zero(b - a, tolerance)
}
pub fn ieee_float_divide(a: f32, b: f32) -> f32 {
    a / b
}

/// Reinterpret the source bytes as `Dst`, matching pinned C++ `bit_cast`.
///
/// # Safety
///
/// Every bit pattern in `source` must be a valid `Dst` value. C++ constrains
/// this operation through its trivially-copyable type system; Rust's `Copy`
/// bound alone does not make arbitrary destination bit patterns valid.
pub unsafe fn bit_cast<Dst: Copy, Src: Copy>(source: Src) -> Dst {
    assert_eq!(size_of::<Dst>(), size_of::<Src>());
    let mut destination = MaybeUninit::<Dst>::uninit();
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&source as *const Src).cast::<u8>(),
            destination.as_mut_ptr().cast::<u8>(),
            size_of::<Src>(),
        );
        unsafe { destination.assume_init() }
    }
}

pub trait Integer: Copy + Eq + Ord + core::fmt::Debug {
    const SIGNED: bool;
    const MIN_I128: i128;
    const MAX_I128: i128;
    fn to_i128(self) -> i128;
    fn from_i128(value: i128) -> Self;
}
macro_rules! signed_integer { ($($ty:ty),* $(,)?) => {$( impl Integer for $ty { const SIGNED: bool = true; const MIN_I128: i128 = <$ty>::MIN as i128; const MAX_I128: i128 = <$ty>::MAX as i128; fn to_i128(self)->i128{self as i128} fn from_i128(value:i128)->Self{value as Self} } )*}; }
macro_rules! unsigned_integer { ($($ty:ty),* $(,)?) => {$( impl Integer for $ty { const SIGNED: bool = false; const MIN_I128: i128 = 0; const MAX_I128: i128 = <$ty>::MAX as i128; fn to_i128(self)->i128{self as i128} fn from_i128(value:i128)->Self{value as Self} } )*}; }
signed_integer!(i8, i16, i32, i64, isize);
unsigned_integer!(u8, u16, u32, u64, usize);

pub fn lossless_numeric_cast<T: Integer, U: Integer>(value: U) -> T {
    let raw = value.to_i128();
    assert!(raw >= T::MIN_I128 && raw <= T::MAX_I128);
    let result = T::from_i128(raw);
    assert_eq!(result.to_i128(), raw);
    if T::SIGNED != U::SIGNED {
        if U::SIGNED {
            assert!(raw >= 0, "lossless_numeric_cast failed due to sign change");
        } else {
            assert!(
                result.to_i128() >= 0,
                "lossless_numeric_cast failed due to sign change"
            );
        }
    }
    result
}
pub fn cmp_equal<A: Integer, B: Integer>(a: A, b: B) -> bool {
    a.to_i128() == b.to_i128()
}
pub fn cmp_not_equal<A: Integer, B: Integer>(a: A, b: B) -> bool {
    !cmp_equal(a, b)
}
pub fn cmp_less<A: Integer, B: Integer>(a: A, b: B) -> bool {
    a.to_i128() < b.to_i128()
}
pub fn cmp_greater<A: Integer, B: Integer>(a: A, b: B) -> bool {
    cmp_less(b, a)
}
pub fn cmp_less_equal<A: Integer, B: Integer>(a: A, b: B) -> bool {
    !cmp_less(b, a)
}
pub fn cmp_greater_equal<A: Integer, B: Integer>(a: A, b: B) -> bool {
    !cmp_less(a, b)
}
pub fn clamp_cast<T: Integer, U: Integer>(value: U) -> T {
    T::from_i128(value.to_i128().clamp(T::MIN_I128, T::MAX_I128))
}

pub trait RoundWord:
    Copy + Add<Output = Self> + BitAnd<Output = Self> + Not<Output = Self>
{
    fn from_usize(value: usize) -> Self;
}
macro_rules! round_word { ($($ty:ty),* $(,)?) => {$(impl RoundWord for $ty { fn from_usize(value:usize)->Self{value as Self} })*}; }
round_word!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
pub fn round_up_to_multiple_of<const N: usize, T: RoundWord>(value: T) -> T {
    assert!(N != 0 && N & (N - 1) == 0);
    (value + T::from_usize(N - 1)) & !T::from_usize(N - 1)
}
pub fn clamp(value: f32, low: f32, high: f32) -> f32 {
    let value = if low < value || low.is_nan() {
        value
    } else {
        low
    };
    if high < value || value.is_nan() {
        high
    } else {
        value
    }
}
pub fn positive_mod(value: f32, mut range: f32) -> f32 {
    if range < 0.0 {
        range = -range;
    }
    let mut result = value % range;
    if result < 0.0 {
        result += range;
    }
    result
}
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}
pub fn padding_to_align_up<const ALIGNMENT: usize>(value: usize) -> u32 {
    let maximum = usize::MAX / ALIGNMENT * ALIGNMENT;
    let padding = (maximum - value) % ALIGNMENT;
    assert_eq!((value + padding) % ALIGNMENT, 0);
    padding as u32
}
pub fn pointer_padding_to_align_up<T>(pointer: *const u8) -> u32 {
    let alignment = core::mem::align_of::<T>();
    let value = pointer as usize;
    let maximum = usize::MAX / alignment * alignment;
    let padding = (maximum - value) % alignment;
    assert_eq!((value + padding) % alignment, 0);
    padding as u32
}
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Copy + Mul<f32, Output = T> + Add<Output = T>,
{
    a * (1.0 - t) + b * t
}
