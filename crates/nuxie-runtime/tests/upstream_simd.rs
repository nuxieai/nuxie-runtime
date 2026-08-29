//! All 23 pinned SIMD cases exercise the translated production SIMD owner.
//! Scalar reference expressions remain only as independent expected results.

use nuxie_runtime::source::math::simd::{self, GVec};
use nuxie_runtime::source::math::simd_gvec_polyfill::compare;

fn all<const N: usize>(values: [bool; N]) -> bool {
    simd::all(GVec::from_array(
        values.map(|value| if value { -1i32 } else { 0 }),
    ))
}

fn any<const N: usize>(values: [bool; N]) -> bool {
    simd::any(GVec::from_array(
        values.map(|value| if value { -1i32 } else { 0 }),
    ))
}

fn min_f32<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    simd::min(GVec::from_array(a), GVec::from_array(b)).data
}

fn max_f32<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    simd::max(GVec::from_array(a), GVec::from_array(b)).data
}

fn clamp(value: f32, low: f32, high: f32) -> f32 {
    simd::clamp(
        GVec::from_array([value]),
        GVec::from_array([low]),
        GVec::from_array([high]),
    )[0]
}

fn dot<const N: usize>(a: [f32; N], b: [f32; N]) -> f32 {
    simd::dot(GVec::from_array(a), GVec::from_array(b))
}

fn fast_acos(value: f32) -> f32 {
    simd::fast_acos(GVec::from_array([value]))[0]
}

fn fuzzy_equal<const N: usize>(a: [f32; N], b: [f32; N]) -> bool {
    a.into_iter().zip(b).all(|(a, b)| (b - a).abs() < 1e-4)
}

#[derive(Clone, Copy)]
struct UpstreamRand(u64);

impl UpstreamRand {
    fn seeded() -> Self {
        Self(u64::from(0_u32.wrapping_sub(1)))
    }

    fn next(&mut self) -> f32 {
        self.0 = 6_364_136_223_846_793_005_u64
            .wrapping_mul(self.0)
            .wrapping_add(1);
        let value = ((self.0 >> 33) as u32) as f32 / 0x7fff_ffff_u32 as f32;
        value.min(f32::from_bits(1.0_f32.to_bits() - 1))
    }
}

#[test]
fn any_case() {
    assert!(!any([false, false, false, false]));
    for lane in 0..4 {
        let values: [bool; 4] = std::array::from_fn(|index| index == lane);
        assert!(any(values));
    }
    assert!(!any([false, false, false]));
    assert!(any([true, false, false]));
    assert!(any([false, true, false]));
    assert!(any([false, false, true]));
    assert!(!any([false, false]));
    assert!(any([true, false]));
    assert!(any([false, true]));
    assert!(!any([false]));
    assert!(any([true]));
}

#[test]
fn all_case() {
    assert!(all([true, true, true, true]));
    for lane in 0..4 {
        let values: [bool; 4] = std::array::from_fn(|index| index != lane);
        assert!(!all(values));
    }
    assert!(all([true, true, true]));
    assert!(!all([false, true, true]));
    assert!(!all([true, false, true]));
    assert!(!all([true, true, false]));
    assert!(all([true, true]));
    assert!(!all([false, true]));
    assert!(!all([true, false]));
    assert!(all([true]));
    assert!(!all([false]));
}

#[test]
fn operators() {
    let a = GVec::from_array([1.0f32, 2.0, 3.0, 4.0]);
    let b = GVec::from_array([5.0f32, 6.0, 7.0, 8.0]);
    assert_eq!((a + b).data, [6.0, 8.0, 10.0, 12.0]);
    assert_eq!((a - b).data, [-4.0; 4]);
    assert_eq!((a * b).data, [5.0, 12.0, 21.0, 32.0]);
    assert_eq!(a / a, b / b);
    assert_eq!(a + 10.0, 10.0 + a);
    // Rust has no unary plus; the unchanged operand is its language adaptation.
    assert_eq!(a - 10.0, -(10.0 - a));
    assert_eq!(a * 2.0, 2.0 * a);
    assert_eq!(a / 0.5, 2.0 * a);
    let i = GVec::from_array([1i32, 2, 4, 8]);
    let j = GVec::from_array([0i32, 1, 3, 7]);
    assert_eq!((i & j).data, [0; 4]);
    assert!(!simd::any(i & j));
    assert_eq!((i | j).data, [1, 3, 7, 15]);
    assert_eq!(i | j, i ^ j);
    assert_eq!(!i + 1, -i);
}

#[test]
fn swizzles() {
    let mut v2 = GVec::from_array([1.0f32, -2.0]);
    assert_eq!(v2.x(), 1.0);
    assert_eq!(v2.y(), -2.0);
    assert_eq!(v2.yx()[0], v2[1]);
    assert_eq!(v2.yx()[1], v2[0]);
    assert_eq!(v2.yx().data, [-2.0, 1.0]);
    assert_eq!(v2.xyxy().data, [1.0, -2.0, 1.0, -2.0]);
    assert_eq!(v2.yxyx().data, [-2.0, 1.0, -2.0, 1.0]);
    let mut v4 = GVec::from_array([1.0f32, -2.0, 3.0, -1.0]);
    assert_eq!((v4.x(), v4.y(), v4.z(), v4.w()), (1.0, -2.0, 3.0, -1.0));
    assert_eq!(v4.xy().data, [1.0, -2.0]);
    assert_eq!(v4.swizzle([1, 2]).data, [-2.0, 3.0]);
    assert_eq!(v4.zw().data, [3.0, -1.0]);
    assert_eq!(v4.xyz().data, [1.0, -2.0, 3.0]);
    assert_eq!(v4.swizzle([1, 2, 3]).data, [-2.0, 3.0, -1.0]);
    assert_eq!(v4.yxwz().data, [-2.0, 1.0, -1.0, 3.0]);
    assert_eq!(v4.zwxy().data, [3.0, -1.0, 1.0, -2.0]);
    assert_eq!(v4.zyxw().data, [3.0, -2.0, 1.0, -1.0]);
    assert_eq!(v4.xwzy().data, [1.0, -1.0, 3.0, -2.0]);
    // Rust's vector owner exposes lane mutation rather than C++ proxy lvalues.
    (v4[0], v4[1]) = (v2.yx()[0], v2.yx()[1]);
    assert_eq!(v4.data, [-2.0, 1.0, 3.0, -1.0]);
    (v4[2], v4[3]) = (v4[1], v4[2]);
    assert_eq!(v4.data, [-2.0, 1.0, 1.0, 3.0]);
    (v4[1], v4[2]) = (-7.0, -7.0);
    assert_eq!(v4.data, [-2.0, -7.0, -7.0, 3.0]);
    (v4[0], v4[1], v4[2]) = (0.0, 0.0, 0.0);
    assert_eq!(v4.data, [0.0, 0.0, 0.0, 3.0]);
    (v4[1], v4[2], v4[3]) = (-9.0, -9.0, -9.0);
    assert_eq!(v4.data, [0.0, -9.0, -9.0, -9.0]);
    (v4[0], v4[1], v4[2], v4[3]) = (1.0, 2.0, -8.0, 0.5);
    assert_eq!(v4.data, [1.0, 2.0, -8.0, 0.5]);
    v2[1] = -9.0;
    v2[0] = 88.0;
    assert_eq!(v2.yx().data, [-9.0, 88.0]);
    let mut v3 = GVec::<i32, 3>::default();
    (v3[0], v3[1], v3[2]) = (0, 9, -1);
    assert_eq!(v3.data, [0, 9, -1]);
    assert_eq!((v3.x(), v3.y(), v3.z()), (0, 9, -1));
    let mut v1 = GVec::<u32, 1>::default();
    v1[0] = 7;
    assert_eq!(v1[0], 7);
    assert_eq!(v1.data, [7]);
    let a = GVec::from_array([0.0f32, 1.0, 12.0, 99.9]);
    let b = GVec::from_array([0.1f32, -1.0, -9.0, -20.0]);
    let a_swizzled = a.yxwz();
    let b_swizzled = b.yxwz();
    assert_eq!(simd::abs(b.yxwz()), simd::abs(b_swizzled));
    assert_eq!(simd::floor(a.yxwz()), simd::floor(a_swizzled));
    assert_eq!(simd::ceil(a.yxwz()), simd::ceil(a_swizzled));
    assert_eq!(simd::sqrt(a.yxwz()), simd::sqrt(a_swizzled));
    assert_eq!(simd::fast_acos(a.yxwz()), simd::fast_acos(a_swizzled));
    assert_eq!(
        simd::min(a.yxwz(), b.yxwz()),
        simd::min(a_swizzled, b_swizzled)
    );
    assert_eq!(
        simd::max(a.yxwz(), b.yxwz()),
        simd::max(a_swizzled, b_swizzled)
    );
    assert_eq!(
        simd::clamp(a.yxwz(), GVec::splat(2.0), GVec::splat(10.0)),
        simd::clamp(a_swizzled, GVec::splat(2.0), GVec::splat(10.0))
    );
    assert_eq!(
        simd::mix(a.yxwz(), b.yxwz(), GVec::splat(0.5)),
        simd::mix(a_swizzled, b_swizzled, GVec::splat(0.5))
    );
    assert_eq!(
        simd::precise_mix(a.yxwz(), b.yxwz(), GVec::splat(0.5)),
        simd::precise_mix(a_swizzled, b_swizzled, GVec::splat(0.5))
    );
    let mask = GVec::from_array([-1i32, 0, 0, 0]);
    assert_eq!(
        simd::if_then_else(mask, a.yxwz(), b.yxwz()),
        simd::if_then_else(mask, a_swizzled, b_swizzled)
    );
    assert_eq!(
        simd::if_then_else(mask, a.yxwz(), b.yxwz()).data,
        [a.y(), b.x(), b.w(), b.z()]
    );
    assert_eq!(
        simd::if_then_else(!mask, a_swizzled, b_swizzled).data,
        [b.y(), a.x(), a.w(), a.z()]
    );
    assert_ne!(
        simd::if_then_else(mask, a.yxwz(), b.yxwz()),
        simd::if_then_else(!mask, a_swizzled, b_swizzled)
    );
    let mut memory = [0.0f32; 4];
    // SAFETY: memory is a live, exclusive four-lane output allocation.
    unsafe { simd::store(memory.as_mut_ptr(), a.yxwz()) };
    assert_eq!(memory, a_swizzled.data);
    // The pinned proxy's default assignment copies its entire backing vector.
    let mut a = a;
    assert_eq!(a, GVec::from_array([0.0f32, 1.0, 12.0, 99.9]));
    a = b;
    assert_eq!(a, b);
}

fn check_ieee<T: Ieee>() {
    T::check();
}

trait Ieee {
    fn check();
}

macro_rules! ieee_impl {
    ($type:ty) => {
        impl Ieee for $type {
            fn check() {
                let infinity = <$type>::INFINITY;
                let v = GVec::from_array([1.0 as $type, -infinity, 1.0, 4.0])
                    / GVec::from_array([0.0, 2.0, infinity, 4.0]);
                assert_eq!(v.data, [infinity, -infinity, 0.0, 1.0]);
                let v = GVec::from_array([infinity, -infinity, infinity, -infinity])
                    * GVec::from_array([infinity, infinity, -infinity, -infinity]);
                assert_eq!(v.data, [infinity, -infinity, -infinity, infinity]);
                let v = GVec::from_array([infinity, -infinity, 0.0, 0.0])
                    / GVec::from_array([0.0, 0.0, infinity, -infinity]);
                assert_eq!(v.data, [infinity, -infinity, 0.0, 0.0]);
                let mut v = GVec::from_array([infinity, 0.0, 0.0, infinity]);
                let xy = v.xy() / GVec::from_array([infinity, 0.0]);
                (v[0], v[1]) = (xy[0], xy[1]);
                v[2] *= infinity;
                v[3] -= infinity;
                for value in v.data {
                    assert!(value.is_nan());
                    assert_ne!(value, value);
                    assert!(!(value <= value));
                    assert!(!(value >= value));
                    assert!(!(value < value));
                    assert!(!(value > value));
                }
                assert!(!simd::any(compare::<$type, 4, i64>(v, v, |a, b| a == b)));
                assert!(simd::all(compare::<$type, 4, i64>(v, v, |a, b| a != b)));
                assert!(!simd::any(compare::<$type, 4, i64>(v, v, |a, b| a <= b)));
                assert!(!simd::any(compare::<$type, 4, i64>(v, v, |a, b| a >= b)));
                assert!(!simd::any(compare::<$type, 4, i64>(v, v, |a, b| a < b)));
                assert!(!simd::any(compare::<$type, 4, i64>(v, v, |a, b| a > b)));
                let v = GVec::from_array([infinity, -infinity, infinity, -infinity])
                    + GVec::from_array([infinity, -infinity, -infinity, infinity]);
                assert_eq!(v.xy().data, [infinity, -infinity]);
                assert!(v[2].is_nan());
                assert!(v[3].is_nan());
            }
        }
    };
}

ieee_impl!(f32);
ieee_impl!(f64);

#[test]
fn ieee_compliance() {
    check_ieee::<f32>();
    check_ieee::<f64>();
}

fn check_ternary<T: Copy + TryFrom<u8> + PartialOrd + std::fmt::Debug>()
where
    T::Error: std::fmt::Debug,
{
    let one = T::try_from(1).unwrap();
    let two = T::try_from(2).unwrap();
    let three = T::try_from(3).unwrap();
    let four = T::try_from(4).unwrap();
    assert_eq!(
        simd::if_then_else(
            GVec::from_array([-1i32, 0]),
            GVec::splat(one),
            GVec::splat(two)
        )
        .data,
        [one, two]
    );
    let mask: GVec<i32, 4> = compare(
        GVec::from_array([one, two, three, four]),
        GVec::from_array([four, three, two, one]),
        |a, b| a < b,
    );
    assert_eq!(
        simd::if_then_else(mask, GVec::splat(one), GVec::splat(two)).data,
        [one, one, two, two]
    );
    assert_eq!(
        simd::if_then_else(
            GVec::from_array([0i32, -1]),
            GVec::from_array([one, two]),
            GVec::from_array([three, four])
        )
        .data,
        [three, two]
    );
    // A scalar condition remains Rust language semantics, as in upstream.
    assert_eq!(
        if one == two {
            GVec::from_array([one, two])
        } else {
            GVec::from_array([three, four])
        }
        .data,
        [three, four]
    );
    let expected = [5u8, 6, 7, 8].map(|value| T::try_from(value).unwrap());
    assert_eq!(
        if three == two {
            GVec::from_array([one, two, three, four])
        } else {
            GVec::from_array(expected)
        }
        .data,
        expected
    );
}

#[test]
fn ternary_operator() {
    check_ternary::<i8>();
    check_ternary::<u8>();
    check_ternary::<i16>();
    check_ternary::<i32>();
    check_ternary::<i64>();
    check_ternary::<u16>();
    check_ternary::<u32>();
    check_ternary::<u64>();
    check_ternary::<usize>();
    check_ternary::<f32>();
    check_ternary::<f64>();
}

#[test]
fn min_max() {
    assert_eq!(
        simd::max(GVec::splat(-1i32), GVec::from_array([-2, 0])).data,
        [-1, 0]
    );
    assert_eq!(
        simd::min(GVec::splat(-1i32), GVec::from_array([-2, 0])).data,
        [-2, -1]
    );
    assert_eq!(
        min_f32([1.0, 2.0, 3.0, 4.0], [4.0, 3.0, 2.0, 0.0]),
        [1.0, 2.0, 2.0, 0.0]
    );
    assert_eq!(
        max_f32([1.0, 2.0, 3.0, 4.0], [4.0, 3.0, 2.0, 0.0]),
        [4.0, 3.0, 3.0, 4.0]
    );
    assert_eq!(
        min_f32(
            [100.0, f32::NEG_INFINITY, f32::NEG_INFINITY, f32::INFINITY],
            [f32::INFINITY, 100.0, f32::INFINITY, f32::NEG_INFINITY]
        ),
        [
            100.0,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY
        ]
    );
    assert_eq!(
        max_f32(
            [100.0, f32::NEG_INFINITY, f32::NEG_INFINITY, f32::INFINITY],
            [f32::INFINITY, 100.0, f32::INFINITY, f32::NEG_INFINITY]
        ),
        [f32::INFINITY, 100.0, f32::INFINITY, f32::INFINITY]
    );
    let minimum = min_f32(
        [1.0, f32::NAN, 2.0, f32::NAN],
        [f32::NAN, 1.0, 1.0, f32::NAN],
    );
    assert_eq!(&minimum[..3], &[1.0, 1.0, 1.0]);
    assert!(minimum[3].is_nan());
    let maximum = max_f32(
        [1.0, f32::NAN, 2.0, f32::NAN],
        [f32::NAN, 1.0, 1.0, f32::NAN],
    );
    assert_eq!(&maximum[..3], &[1.0, 1.0, 2.0]);
    assert!(maximum[3].is_nan());
    for value in [-2.0, f32::NEG_INFINITY, f32::INFINITY] {
        assert_eq!(min_f32([f32::NAN], [value])[0], value);
        assert_eq!(max_f32([f32::NAN], [value])[0], value);
        assert_eq!(min_f32([value], [f32::NAN])[0], value);
        assert_eq!(max_f32([value], [f32::NAN])[0], value);
        assert_eq!(f32::NAN.min(value), value);
        assert_eq!(f32::NAN.max(value), value);
    }
    for value in [1.0, f32::NEG_INFINITY, f32::INFINITY] {
        assert_eq!(
            simd::min(GVec::from_array([value]), GVec::from_array([f32::NAN]))[0],
            value
        );
        assert_eq!(
            simd::max(GVec::from_array([value]), GVec::from_array([f32::NAN]))[0],
            value
        );
        assert_eq!(value.min(f32::NAN), value);
        assert_eq!(value.max(f32::NAN), value);
    }
    for value in [-1.0, f64::NEG_INFINITY, f64::INFINITY] {
        assert_eq!(
            simd::min(GVec::from_array([f64::NAN]), GVec::from_array([value]))[0],
            value
        );
        assert_eq!(
            simd::max(GVec::from_array([f64::NAN]), GVec::from_array([value]))[0],
            value
        );
        assert_eq!(f64::NAN.min(value), value);
        assert_eq!(f64::NAN.max(value), value);
    }
    for value in [2.0, f64::NEG_INFINITY, f64::INFINITY] {
        assert_eq!(
            simd::min(GVec::from_array([value]), GVec::from_array([f64::NAN]))[0],
            value
        );
        assert_eq!(
            simd::max(GVec::from_array([value]), GVec::from_array([f64::NAN]))[0],
            value
        );
        assert_eq!(value.min(f64::NAN), value);
        assert_eq!(value.max(f64::NAN), value);
    }
    assert_eq!(
        simd::max(
            GVec::from_array([3.0f64, 4.0]),
            GVec::from_array([4.0, 3.0])
        )
        .data,
        [4.0, 4.0]
    );
    assert_eq!(
        simd::min(GVec::from_array([3u64, 4]), GVec::from_array([4, 3])).data,
        [3, 3]
    );
    assert_eq!(
        simd::max(GVec::from_array([3usize, 4]), GVec::from_array([4, 3])).data,
        [4, 4]
    );
    assert_eq!(
        simd::max(
            GVec::from_array(std::array::from_fn::<_, 16, _>(|i| i as u8)),
            GVec::from_array(std::array::from_fn(|i| (15 - i) as u8))
        )
        .data,
        [15, 14, 13, 12, 11, 10, 9, 8, 8, 9, 10, 11, 12, 13, 14, 15]
    );
}

#[test]
fn clamp_case() {
    assert_eq!(clamp(1.0, 2.0, 3.0), 2.0);
    assert_eq!(clamp(2.0, 1.0, 1.0), 1.0);
    assert_eq!(
        clamp(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        f32::INFINITY
    );
    assert_eq!(clamp(f32::NEG_INFINITY, 0.0, f32::INFINITY), 0.0);
    assert_eq!(clamp(f32::NAN, 1.0, 2.0), 1.0);
    assert_eq!(clamp(3.0, 2.0, 1.0), 1.0);
    assert_eq!(clamp(f32::NAN, 2.0, 1.0), 1.0);
    assert_eq!(clamp(f32::NAN, f32::NAN, 1.0), 1.0);
    assert_eq!(clamp(3.0, 4.0, f32::NAN), 4.0);
    assert_eq!(clamp(3.0, 2.0, f32::NAN), 3.0);
    assert_eq!(clamp(3.0, f32::NAN, 2.0), 2.0);
    assert_eq!(clamp(3.0, f32::NAN, 4.0), 3.0);
    assert_eq!(clamp(3.0, f32::NAN, f32::NAN), 3.0);
}

#[test]
fn abs_case() {
    assert_eq!(
        simd::abs(GVec::from_array([-1.0_f32, 2.0, -3.0, 4.0])).data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::abs(GVec::from_array([-5.0_f32, 6.0])).data,
        [5.0, 6.0]
    );
    assert_eq!(
        simd::abs(GVec::from_array([-0.0_f32, 0.0])).data,
        [0.0, 0.0]
    );
    assert!(simd::abs(GVec::from_array([f32::NAN]))[0].is_nan());
    assert!(simd::abs(GVec::from_array([(-f32::NAN)]))[0].is_nan());
    assert_eq!(
        simd::abs(GVec::from_array([7_i32, -8, 9, -10])).data,
        [7, 8, 9, 10]
    );
    assert_eq!(simd::abs(GVec::from_array([i32::MIN]))[0], i32::MIN);
}

#[test]
fn reduce() {
    macro_rules! arithmetic {
        ($ty:ty) => {{
            let v = GVec::from_array([1 as $ty, 2 as $ty, 3 as $ty, 4 as $ty]);
            assert_eq!(simd::reduce_add(v), 10 as $ty);
            assert_eq!(simd::reduce_add(v.zwxy()), 10 as $ty);
            assert_eq!(simd::reduce_add(v.xyz()), 6 as $ty);
            assert_eq!(simd::reduce_add(v.swizzle([1, 2])), 5 as $ty);
            assert_eq!(simd::reduce_add(v.xy().yxyx()), 6 as $ty);
            assert_eq!(simd::reduce_min(v), 1 as $ty);
            assert_eq!(simd::reduce_min(v.zwxy()), 1 as $ty);
            assert_eq!(simd::reduce_min(v.xyz()), 1 as $ty);
            assert_eq!(simd::reduce_min(v.swizzle([1, 2])), 2 as $ty);
            assert_eq!(simd::reduce_min(v.xy().yxyx()), 1 as $ty);
            assert_eq!(simd::reduce_max(v), 4 as $ty);
            assert_eq!(simd::reduce_max(v.zwxy()), 4 as $ty);
            assert_eq!(simd::reduce_max(v.xyz()), 3 as $ty);
            assert_eq!(simd::reduce_max(v.swizzle([1, 2])), 3 as $ty);
            assert_eq!(simd::reduce_max(v.xy().yxyx()), 2 as $ty);
            v
        }};
    }
    macro_rules! bitwise {
        ($v:expr) => {{
            let v = $v;
            assert_eq!(simd::reduce_and(v), 0);
            assert_eq!(simd::reduce_and(v.zwxy()), 0);
            assert_eq!(simd::reduce_and(v.xyz()), 0);
            assert_eq!(simd::reduce_and(v.swizzle([1, 2])), 2);
            assert_eq!(simd::reduce_and(v.xy().yxyx()), 0);
            assert_eq!(simd::reduce_or(v), 7);
            assert_eq!(simd::reduce_or(v.zwxy()), 7);
            assert_eq!(simd::reduce_or(v.xyz()), 3);
            assert_eq!(simd::reduce_or(v.swizzle([1, 2])), 3);
            assert_eq!(simd::reduce_or(v.xy().yxyx()), 3);
        }};
    }
    arithmetic!(f32);
    bitwise!(arithmetic!(i32));
    bitwise!(arithmetic!(u32));
}

#[test]
fn floor() {
    assert_eq!(
        simd::floor(GVec::from_array([-1.9_f32, 1.9, 2.0, -2.0])).data,
        [-2.0, 1.0, 2.0, -2.0]
    );
    assert_eq!(
        simd::floor(GVec::from_array([f32::INFINITY, f32::NEG_INFINITY])).data,
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(simd::floor(GVec::from_array([f32::NAN]))[0].is_nan());
    assert!(simd::floor(GVec::from_array([(-f32::NAN)]))[0].is_nan());
}

#[test]
fn ceil() {
    assert_eq!(
        simd::ceil(GVec::from_array([-1.9_f32, 1.9, 2.0, -2.0])).data,
        [-1.0, 2.0, 2.0, -2.0]
    );
    assert_eq!(
        simd::ceil(GVec::from_array([f32::INFINITY, f32::NEG_INFINITY])).data,
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(simd::ceil(GVec::from_array([f32::NAN]))[0].is_nan());
    assert!(simd::ceil(GVec::from_array([(-f32::NAN)]))[0].is_nan());
}

#[test]
fn copysign() {
    let values = [-1.0_f32, 2.0, -3.0, 4.0];
    let signs = [-999.2_f32, f32::NEG_INFINITY, 123.4, 0.0000001];
    assert_eq!(
        simd::copy_sign(GVec::from_array(values), GVec::from_array(signs)).data,
        [-1.0, -2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::copy_sign(
            GVec::from_array([998.0f32, -23.0]),
            GVec::from_array([-1.0, 1.0])
        )
        .data,
        [-998.0, 23.0]
    );
    assert!(simd::copy_sign(GVec::from_array([f32::NAN]), GVec::from_array([-1.0]))[0].is_nan());
    assert!(simd::copy_sign(GVec::from_array([-f32::NAN]), GVec::from_array([1.0]))[0].is_nan());
}

#[test]
fn sqrt() {
    assert_eq!(
        simd::sqrt(GVec::from_array([1.0_f32, 4.0, 9.0, 16.0])).data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::sqrt(GVec::from_array([49.0_f32, 64.0, 81.0, 100.0, 121.0])).data,
        [7.0, 8.0, 9.0, 10.0, 11.0]
    );
    for value in [-1.0, f32::NEG_INFINITY, f32::NAN, -2.0] {
        assert!(simd::sqrt(GVec::from_array([value]))[0].is_nan());
    }
    assert_eq!(
        simd::sqrt(GVec::from_array([f32::INFINITY, 0.0, 1.0])).data,
        [f32::INFINITY, 0.0, 1.0]
    );
}

#[test]
fn div255() {
    for base in (0..255 * 255).step_by(8) {
        let values = std::array::from_fn::<_, 8, _>(|lane| (base + lane).min(255 * 255) as u16);
        assert_eq!(
            simd::div255(GVec::from_array(values)).data,
            values.map(|value| ((u32::from(value) + 127) / 255) as u16)
        );
    }
}

#[test]
fn fast_acos_case() {
    const MAX_ERROR: f32 = 0.016_755_2;
    for value in [-1.0_f32, 0.0, 1.0] {
        assert!((value.acos() - fast_acos(value)).abs() <= MAX_ERROR);
    }
    let mut x = [-0.99_f32, -0.8, -0.4, -0.2, 0.2, 0.4, 0.8, 0.99];
    let mut derivative = [0.0; 8];
    for _ in 0..10 {
        for lane in 0..8 {
            assert!((x[lane].acos() - fast_acos(x[lane])).abs() <= MAX_ERROR);
            let square = x[lane] * x[lane];
            let a = -0.939_115_6;
            let b = 0.921_784_16;
            let c = -1.284_590_6;
            let d = 0.295_624_14;
            let f = (b * square + a) * x[lane];
            let f1 = 3.0 * b * square + a;
            let g = (d * square + c) * square + 1.0;
            let g1 = (4.0 * d * square + 2.0 * c) * x[lane];
            let q = (1.0 - square).sqrt();
            derivative[lane] = (f1 * g - f * g1) / (g * g) + 1.0 / q;
            let f2 = 6.0 * b * x[lane];
            let g2 = 12.0 * d * square + 2.0 * c;
            let second = ((f2 * g - f * g2) * g - (f1 * g - f * g1) * 2.0 * g1) / (g * g * g)
                + x[lane] / ((1.0 - square) * q);
            x[lane] = clamp(x[lane] - derivative[lane] / second, -0.99, 0.99);
        }
    }
    assert!(derivative.into_iter().all(|value| value.abs() < 1e-4));
    for root in [
        -0.983_536, -0.867_381, -0.410_923, 0.410_923, 0.867_381, 0.983_536,
    ] {
        assert!(x.into_iter().any(|value| (value - root).abs() < 1e-4));
    }
}

#[test]
fn cast() {
    let values = GVec::from_array([-1.9f32, -1.5, 1.5, 1.1]);
    assert_eq!(simd::cast::<i32, _, 4>(values).data, [-1, -1, 1, 1]);
    assert_eq!(
        simd::cast::<i32, _, 4>(simd::floor(values)).data,
        [-2, -2, 1, 1]
    );
    assert_eq!(
        simd::cast::<i32, _, 4>(simd::ceil(values)).data,
        [-1, -1, 2, 2]
    );
    assert_eq!(
        simd::cast::<i32, _, 4>(simd::ceil(values.zwxy())).data,
        [2, 2, -1, -1]
    );
}

#[test]
fn dot_case() {
    assert_eq!(dot([0.0, 1.0], [1.0, 0.0]), 0.0);
    assert_eq!(dot([1.0, 1.0], [1.0, -1.0]), 0.0);
    assert_eq!(dot([1.0, 2.0, -3.0], [1.0, 2.0, 3.0]), -4.0);
    assert_eq!(dot([1.0, 2.0, 3.0, 4.0], [1.0, 2.0, 3.0, 4.0]), 30.0);
    assert_eq!(
        dot([1.0, 2.0, 3.0, 4.0, 5.0], [1.0, 2.0, 3.0, 4.0, -5.0]),
        5.0
    );
    assert_eq!(dot([1.0, 2.0, 3.0, 4.0], [4.0, 3.0, 2.0, 1.0]), 20.0);
    assert_eq!(
        dot([1.0, 2.0, 3.0, 4.0, 5.0], [1.0, 2.0, 3.0, 4.0, 5.0]),
        55.0
    );
}

#[test]
fn cross() {
    assert_eq!(
        simd::cross(GVec::from_array([0.0, 1.0]), GVec::from_array([0.0, 1.0])),
        0.0
    );
    assert_eq!(
        simd::cross(GVec::from_array([1.0, 0.0]), GVec::from_array([1.0, 0.0])),
        0.0
    );
    assert_eq!(
        simd::cross(GVec::from_array([1.0, 1.0]), GVec::from_array([1.0, 1.0])),
        0.0
    );
    assert_eq!(
        simd::cross(GVec::from_array([1.0, 1.0]), GVec::from_array([1.0, -1.0])),
        -2.0
    );
    assert_eq!(
        simd::cross(GVec::from_array([1.0, 1.0]), GVec::from_array([-1.0, 1.0])),
        2.0
    );
}

#[test]
fn join() {
    assert_eq!(
        simd::join::<_, 2, 4, 6>(GVec::from_array([1, 2]), GVec::from_array([3, 4, 5, 6])).data,
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        simd::join::<_, 1, 3, 4>(
            GVec::from_array([1.0f32]),
            GVec::from_array([2.0, 3.0, 4.0])
        )
        .data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::join3::<_, 1, 2, 3, 6>(
            GVec::from_array([1]),
            GVec::from_array([2, 3]),
            GVec::from_array([4, 5, 6])
        )
        .data,
        [1, 2, 3, 4, 5, 6]
    );
    let joined = simd::join4::<_, 8, 8, 8, 8, 32>(
        GVec::splat(3u8),
        GVec::splat(9),
        GVec::splat(3),
        GVec::splat(100),
    )
    .data;
    assert_eq!(&joined[..8], &[3; 8]);
    assert_eq!(&joined[8..16], &[9; 8]);
    assert_eq!(&joined[16..24], &[3; 8]);
    assert_eq!(&joined[24..], &[100; 8]);
}

fn zip<T: Copy + Default, const N: usize, const OUT: usize>(a: [T; N], b: [T; N]) -> [T; OUT] {
    simd::zip::<T, N, OUT>(GVec::from_array(a), GVec::from_array(b)).data
}

#[test]
fn zip_case() {
    assert_eq!(zip::<_, 1, 2>(['a'], ['b']), ['a', 'b']);
    assert_eq!(zip::<_, 2, 4>([1, 2], [3, 4]), [1, 3, 2, 4]);
    assert_eq!(
        zip::<_, 4, 8>([1, 2, 3, 4], [5, 6, 7, 8]),
        [1, 5, 2, 6, 3, 7, 4, 8]
    );
    assert_eq!(
        zip::<_, 8, 16>([1_u8, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]),
        [1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15, 8, 16]
    );
}

fn check_mix<const N: usize>(random: &mut UpstreamRand) {
    let a: [f32; N] = std::array::from_fn(|_| random.next());
    let b: [f32; N] = std::array::from_fn(|_| random.next());
    let scalar = random.next();
    let expected = std::array::from_fn(|index| a[index] * (1.0 - scalar) + b[index] * scalar);
    assert!(fuzzy_equal(
        simd::mix(
            GVec::from_array(a),
            GVec::from_array(b),
            GVec::splat(scalar)
        )
        .data,
        expected
    ));
    assert!(fuzzy_equal(
        simd::precise_mix(
            GVec::from_array(a),
            GVec::from_array(b),
            GVec::splat(scalar)
        )
        .data,
        expected
    ));
    let lanes: [f32; N] = std::array::from_fn(|_| random.next());
    let expected =
        std::array::from_fn(|index| a[index] * (1.0 - lanes[index]) + b[index] * lanes[index]);
    assert!(fuzzy_equal(
        simd::mix(
            GVec::from_array(a),
            GVec::from_array(b),
            GVec::from_array(lanes)
        )
        .data,
        expected
    ));
    assert!(fuzzy_equal(
        simd::precise_mix(
            GVec::from_array(a),
            GVec::from_array(b),
            GVec::from_array(lanes)
        )
        .data,
        expected
    ));
}

#[test]
fn mix() {
    let mut random = UpstreamRand::seeded();
    check_mix::<1>(&mut random);
    check_mix::<2>(&mut random);
    check_mix::<3>(&mut random);
    check_mix::<4>(&mut random);
    check_mix::<5>(&mut random);
    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];
    assert_eq!(
        simd::mix(GVec::from_array(a), GVec::from_array(b), GVec::splat(0.0)).data,
        a
    );
    assert_eq!(
        simd::precise_mix(GVec::from_array(a), GVec::from_array(b), GVec::splat(1.0)).data,
        b
    );
    assert_eq!(
        simd::precise_mix(
            GVec::from_array([-1.0, 2.0, 3.0, 4.0]),
            GVec::from_array(b),
            GVec::splat(0.0)
        )
        .data,
        [-1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::precise_mix(
            GVec::from_array(a),
            GVec::from_array([5.0, -6.0, 7.0, -8.0]),
            GVec::splat(1.0)
        )
        .data,
        [5.0, -6.0, 7.0, -8.0]
    );
}

#[test]
fn load4x4f() {
    let matrix = [
        0.0, 4.0, 8.0, 12.0, 1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0,
    ];
    let (a, b, c, d) = simd::load4x4f(&matrix);
    let rows = [a.data, b.data, c.data, d.data];
    assert_eq!(rows[0], [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(rows[1], [4.0, 5.0, 6.0, 7.0]);
    assert_eq!(rows[2], [8.0, 9.0, 10.0, 11.0]);
    assert_eq!(rows[3], [12.0, 13.0, 14.0, 15.0]);
}
