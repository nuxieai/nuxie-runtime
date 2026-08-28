pub use super::simd_gvec_polyfill::GVec;
use core::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};

pub type Float2 = GVec<f32, 2>;
pub type Float4 = GVec<f32, 4>;
pub type Int2 = GVec<i32, 2>;
pub type Int4 = GVec<i32, 4>;
pub type Uint2 = GVec<u32, 2>;
pub type Uint4 = GVec<u32, 4>;
pub type Int8x8 = GVec<i8, 8>;
pub type Int8x16 = GVec<i8, 16>;
pub type Int8x32 = GVec<i8, 32>;
pub type Uint8x8 = GVec<u8, 8>;
pub type Uint8x16 = GVec<u8, 16>;
pub type Uint8x32 = GVec<u8, 32>;
pub type Int16x4 = GVec<i16, 4>;
pub type Int16x8 = GVec<i16, 8>;
pub type Int16x16 = GVec<i16, 16>;
pub type Uint16x4 = GVec<u16, 4>;
pub type Uint16x8 = GVec<u16, 8>;
pub type Uint16x16 = GVec<u16, 16>;
pub type Int64x2 = GVec<i64, 2>;
pub type Int64x4 = GVec<i64, 4>;
pub type Uint64x2 = GVec<u64, 2>;
pub type Uint64x4 = GVec<u64, 4>;

pub trait Truthy {
    fn truthy(self) -> bool;
}
macro_rules! truthy{($($ty:ty),*$(,)?)=>{$(impl Truthy for $ty{fn truthy(self)->bool{self!=0}})*};}
truthy!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
pub fn any<T: Copy + Truthy, const N: usize>(x: GVec<T, N>) -> bool {
    x.data.into_iter().any(Truthy::truthy)
}
pub fn all<T: Copy + Truthy + NotBits, const N: usize>(x: GVec<T, N>) -> bool {
    !any(GVec {
        data: x.data.map(NotBits::not_bits),
    })
}
pub trait NotBits {
    fn not_bits(self) -> Self;
}
macro_rules! not_bits{($($ty:ty),*$(,)?)=>{$(impl NotBits for $ty{fn not_bits(self)->Self{!self}})*};}
not_bits!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
pub fn isnan<const N: usize>(x: GVec<f32, N>) -> GVec<i32, N> {
    GVec {
        data: x.data.map(|value| if value.is_nan() { -1 } else { 0 }),
    }
}
pub fn if_then_else<T: Copy, M: Truthy, const N: usize>(
    mask: GVec<M, N>,
    then_values: GVec<T, N>,
    else_values: GVec<T, N>,
) -> GVec<T, N> {
    GVec {
        data: core::array::from_fn(|i| {
            if mask[i].truthy() {
                then_values[i]
            } else {
                else_values[i]
            }
        }),
    }
}
pub trait SimdMinMax: Copy + PartialOrd {
    fn simd_min(self, other: Self) -> Self {
        if other < self {
            other
        } else {
            self
        }
    }
    fn simd_max(self, other: Self) -> Self {
        if self < other {
            other
        } else {
            self
        }
    }
}
impl SimdMinMax for f32 {
    fn simd_min(self, other: Self) -> Self {
        if self.is_nan() {
            other
        } else if other.is_nan() {
            self
        } else if self == 0.0 && other == 0.0 {
            Self::from_bits(self.to_bits() | other.to_bits())
        } else if other < self {
            other
        } else {
            self
        }
    }
    fn simd_max(self, other: Self) -> Self {
        if self.is_nan() {
            other
        } else if other.is_nan() {
            self
        } else if self == 0.0 && other == 0.0 {
            Self::from_bits(self.to_bits() & other.to_bits())
        } else if self < other {
            other
        } else {
            self
        }
    }
}
macro_rules! minmax{($($ty:ty),*$(,)?)=>{$(impl SimdMinMax for $ty{})*};}
minmax!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
pub fn min<T: SimdMinMax, const N: usize>(a: GVec<T, N>, b: GVec<T, N>) -> GVec<T, N> {
    GVec {
        data: core::array::from_fn(|i| a[i].simd_min(b[i])),
    }
}
pub fn max<T: SimdMinMax, const N: usize>(a: GVec<T, N>, b: GVec<T, N>) -> GVec<T, N> {
    GVec {
        data: core::array::from_fn(|i| a[i].simd_max(b[i])),
    }
}
pub fn clamp<T: SimdMinMax, const N: usize>(
    x: GVec<T, N>,
    low: GVec<T, N>,
    high: GVec<T, N>,
) -> GVec<T, N> {
    min(max(low, x), high)
}
pub trait SimdAbs: Copy {
    fn simd_abs(self) -> Self;
}
impl SimdAbs for f32 {
    fn simd_abs(self) -> Self {
        self.abs()
    }
}
macro_rules! abs_signed{($($ty:ty),*$(,)?)=>{$(impl SimdAbs for $ty{fn simd_abs(self)->Self{if self<0{self.wrapping_neg()}else{self}}})*};}
abs_signed!(i8, i16, i32, i64, i128, isize);
macro_rules! abs_unsigned{($($ty:ty),*$(,)?)=>{$(impl SimdAbs for $ty{fn simd_abs(self)->Self{self}})*};}
abs_unsigned!(u8, u16, u32, u64, u128, usize);
pub fn abs<T: SimdAbs, const N: usize>(x: GVec<T, N>) -> GVec<T, N> {
    GVec {
        data: x.data.map(SimdAbs::simd_abs),
    }
}
pub fn reduce_add<T: Copy + Add<Output = T>, const N: usize>(x: GVec<T, N>) -> T {
    x.data[1..].iter().copied().fold(x[0], |a, b| a + b)
}
pub fn reduce_min<T: Copy + PartialOrd, const N: usize>(x: GVec<T, N>) -> T {
    x.data[1..]
        .iter()
        .copied()
        .fold(x[0], |a, b| if b < a { b } else { a })
}
pub fn reduce_max<T: Copy + PartialOrd, const N: usize>(x: GVec<T, N>) -> T {
    x.data[1..]
        .iter()
        .copied()
        .fold(x[0], |a, b| if a < b { b } else { a })
}
pub fn reduce_and<T: Copy + BitAnd<Output = T>, const N: usize>(x: GVec<T, N>) -> T {
    x.data[1..].iter().copied().fold(x[0], |a, b| a & b)
}
pub fn reduce_or<T: Copy + BitOr<Output = T>, const N: usize>(x: GVec<T, N>) -> T {
    x.data[1..].iter().copied().fold(x[0], |a, b| a | b)
}
pub fn floor<const N: usize>(x: GVec<f32, N>) -> GVec<f32, N> {
    GVec {
        data: x.data.map(f32::floor),
    }
}
pub fn ceil<const N: usize>(x: GVec<f32, N>) -> GVec<f32, N> {
    GVec {
        data: x.data.map(f32::ceil),
    }
}
pub fn copy_sign<const N: usize>(x: GVec<f32, N>, y: GVec<f32, N>) -> GVec<f32, N> {
    GVec {
        data: core::array::from_fn(|i| x[i].copysign(y[i])),
    }
}
pub fn sqrt<const N: usize>(x: GVec<f32, N>) -> GVec<f32, N> {
    GVec {
        data: x.data.map(f32::sqrt),
    }
}
pub fn div255<const N: usize>(mut x: GVec<u16, N>) -> GVec<u16, N> {
    assert!(x.data.iter().all(|v| *v <= 255 * 255));
    x += 128;
    GVec {
        data: x.data.map(|v| (v + (v >> 8)) >> 8),
    }
}
pub const FAST_ACOS_MAX_ERROR: f32 = 0.0167552;
pub fn fast_acos<const N: usize>(x: GVec<f32, N>) -> GVec<f32, N> {
    const A: f32 = -0.9391156;
    const B: f32 = 0.92178416;
    const C: f32 = -1.2845906;
    const D: f32 = 0.29562414;
    const HALF_PI: f32 = 1.5707964;
    GVec {
        data: x.data.map(|v| {
            let xx = v * v;
            v * ((B * xx + A) / (xx * (D * xx + C) + 1.0)) + HALF_PI
        }),
    }
}
pub trait CastFrom<T> {
    fn cast_from(value: T) -> Self;
}
macro_rules! casts{($dst:ty;$($src:ty),*$(,)?)=>{$(impl CastFrom<$src> for $dst{fn cast_from(value:$src)->Self{value as Self}})*};}
casts!(f32;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(i32;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(u32;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(u16;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(u8;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(i16;i8,i16,i32,i64,u8,u16,u32,u64,f32);
casts!(i8;i8,i16,i32,i64,u8,u16,u32,u64,f32);
pub fn cast<U: CastFrom<T>, T: Copy, const N: usize>(x: GVec<T, N>) -> GVec<U, N> {
    GVec {
        data: core::array::from_fn(|i| U::cast_from(x[i])),
    }
}
/// Load `N` contiguous SIMD lanes from a call-scoped foreign buffer.
///
/// # Safety
///
/// `pointer` must be non-null, properly aligned, and readable for `N`
/// initialized `T` values in one live allocation. The returned vector copies
/// the values and does not retain the pointer.
pub unsafe fn load<T: Copy, const N: usize>(pointer: *const T) -> GVec<T, N> {
    GVec {
        // SAFETY: the caller guarantees the complete `0..N` input range.
        data: core::array::from_fn(|i| unsafe { *pointer.add(i) }),
    }
}

/// Store `N` contiguous SIMD lanes into a call-scoped foreign buffer.
///
/// # Safety
///
/// `pointer` must be non-null, properly aligned, and writable for `N` `T`
/// values in one live allocation. It must not alias a live Rust reference for
/// the duration of this call. No pointer is retained.
pub unsafe fn store<T: Copy, const N: usize>(pointer: *mut T, vector: GVec<T, N>) {
    for i in 0..N {
        // SAFETY: the caller guarantees the complete `0..N` output range and
        // exclusive access for this call.
        unsafe {
            *pointer.add(i) = vector[i];
        }
    }
}
pub fn load4x4f(matrix: &[f32; 16]) -> (Float4, Float4, Float4, Float4) {
    (
        GVec::from_array([matrix[0], matrix[4], matrix[8], matrix[12]]),
        GVec::from_array([matrix[1], matrix[5], matrix[9], matrix[13]]),
        GVec::from_array([matrix[2], matrix[6], matrix[10], matrix[14]]),
        GVec::from_array([matrix[3], matrix[7], matrix[11], matrix[15]]),
    )
}
pub fn join<T: Copy + Default, const A: usize, const B: usize, const OUT: usize>(
    a: GVec<T, A>,
    b: GVec<T, B>,
) -> GVec<T, OUT> {
    assert_eq!(OUT, A + B);
    let mut output = GVec::default();
    output.data[..A].copy_from_slice(&a.data);
    output.data[A..].copy_from_slice(&b.data);
    output
}
pub fn join3<
    T: Copy + Default,
    const A: usize,
    const B: usize,
    const C: usize,
    const OUT: usize,
>(
    a: GVec<T, A>,
    b: GVec<T, B>,
    c: GVec<T, C>,
) -> GVec<T, OUT> {
    assert_eq!(OUT, A + B + C);
    let mut output = GVec::default();
    output.data[..A].copy_from_slice(&a.data);
    output.data[A..A + B].copy_from_slice(&b.data);
    output.data[A + B..].copy_from_slice(&c.data);
    output
}
pub fn join4<
    T: Copy + Default,
    const A: usize,
    const B: usize,
    const C: usize,
    const D: usize,
    const OUT: usize,
>(
    a: GVec<T, A>,
    b: GVec<T, B>,
    c: GVec<T, C>,
    d: GVec<T, D>,
) -> GVec<T, OUT> {
    assert_eq!(OUT, A + B + C + D);
    let mut output = GVec::default();
    output.data[..A].copy_from_slice(&a.data);
    output.data[A..A + B].copy_from_slice(&b.data);
    output.data[A + B..A + B + C].copy_from_slice(&c.data);
    output.data[A + B + C..].copy_from_slice(&d.data);
    output
}
pub fn zip<T: Copy + Default, const N: usize, const OUT: usize>(
    a: GVec<T, N>,
    b: GVec<T, N>,
) -> GVec<T, OUT> {
    assert_eq!(OUT, N * 2);
    let mut output = GVec::default();
    for i in 0..N {
        output[i * 2] = a[i];
        output[i * 2 + 1] = b[i];
    }
    output
}
pub fn dot<T: Copy + Mul<Output = T> + Add<Output = T>, const N: usize>(
    a: GVec<T, N>,
    b: GVec<T, N>,
) -> T {
    reduce_add(a * b)
}
pub fn cross(a: Float2, b: Float2) -> f32 {
    let c = a * b.yx();
    c[0] - c[1]
}
pub fn mul_add<const N: usize>(
    a: GVec<f32, N>,
    b: GVec<f32, N>,
    addend: GVec<f32, N>,
) -> GVec<f32, N> {
    GVec {
        data: core::array::from_fn(|i| a[i].mul_add(b[i], addend[i])),
    }
}
pub fn mix<const N: usize>(a: GVec<f32, N>, b: GVec<f32, N>, t: GVec<f32, N>) -> GVec<f32, N> {
    assert!(t.data.iter().all(|v| *v >= 0.0 && *v < 1.0));
    mul_add(b - a, t, a)
}
pub fn unchecked_mix<const N: usize>(
    a: GVec<f32, N>,
    b: GVec<f32, N>,
    t: GVec<f32, N>,
) -> GVec<f32, N> {
    mul_add(b - a, t, a)
}
pub fn precise_mix<const N: usize>(
    a: GVec<f32, N>,
    b: GVec<f32, N>,
    t: GVec<f32, N>,
) -> GVec<f32, N> {
    mul_add(a, GVec::splat(1.0) - t, b * t)
}
