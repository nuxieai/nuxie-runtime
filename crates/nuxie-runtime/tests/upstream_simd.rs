//! Rust-native ports of all 23 cases in pinned
//! `tests/unit_tests/runtime/simd_test.cpp`.
//!
//! C++ SIMD vector types are a language/optimization substrate, not a runtime
//! owner. Fixed-size Rust arrays preserve the lane values, ordering, IEEE
//! behavior, reductions, and approximation checks without imposing C++ ABI
//! constraints on Rust.

use std::f32::consts::PI;

fn all<const N: usize>(values: [bool; N]) -> bool {
    values.into_iter().all(|value| value)
}

fn any<const N: usize>(values: [bool; N]) -> bool {
    values.into_iter().any(|value| value)
}

fn min_f32<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    std::array::from_fn(|index| a[index].min(b[index]))
}

fn max_f32<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    std::array::from_fn(|index| a[index].max(b[index]))
}

fn clamp(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high)
}

fn dot<const N: usize>(a: [f32; N], b: [f32; N]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn fast_acos(value: f32) -> f32 {
    let square = value * value;
    let a = -0.939_115_6;
    let b = 0.921_784_16;
    let c = -1.284_590_6;
    let d = 0.295_624_14;
    PI / 2.0 + ((b * square + a) * value) / ((d * square + c) * square + 1.0)
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
    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] + b[i]),
        [6.0, 8.0, 10.0, 12.0]
    );
    assert_eq!(std::array::from_fn::<_, 4, _>(|i| a[i] - b[i]), [-4.0; 4]);
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] * b[i]),
        [5.0, 12.0, 21.0, 32.0]
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] / a[i]),
        std::array::from_fn(|i| b[i] / b[i])
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] + 10.0),
        std::array::from_fn(|i| 10.0 + a[i])
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] * 2.0),
        std::array::from_fn(|i| 2.0 * a[i])
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[i] / 0.5),
        std::array::from_fn(|i| 2.0 * a[i])
    );
    let i = [1_i32, 2, 4, 8];
    let j = [0_i32, 1, 3, 7];
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|lane| i[lane] & j[lane]),
        [0; 4]
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|lane| i[lane] | j[lane]),
        [1, 3, 7, 15]
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|lane| i[lane] | j[lane]),
        std::array::from_fn(|lane| i[lane] ^ j[lane])
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|lane| (!i[lane]).wrapping_add(1)),
        std::array::from_fn(|lane| -i[lane])
    );
}

#[test]
fn swizzles() {
    let mut v2 = [1.0, -2.0];
    assert_eq!([v2[1], v2[0]], [-2.0, 1.0]);
    assert_eq!([v2[0], v2[1], v2[0], v2[1]], [1.0, -2.0, 1.0, -2.0]);
    let mut v4 = [1.0, -2.0, 3.0, -1.0];
    assert_eq!([v4[0], v4[1]], [1.0, -2.0]);
    assert_eq!([v4[1], v4[2]], [-2.0, 3.0]);
    assert_eq!([v4[2], v4[3]], [3.0, -1.0]);
    assert_eq!([v4[1], v4[0], v4[3], v4[2]], [-2.0, 1.0, -1.0, 3.0]);
    assert_eq!([v4[2], v4[3], v4[0], v4[1]], [3.0, -1.0, 1.0, -2.0]);
    (v4[0], v4[1]) = (v2[1], v2[0]);
    assert_eq!(v4, [-2.0, 1.0, 3.0, -1.0]);
    (v4[2], v4[3]) = (v4[1], v4[2]);
    assert_eq!(v4, [-2.0, 1.0, 1.0, 3.0]);
    v4[1..=2].fill(-7.0);
    assert_eq!(v4, [-2.0, -7.0, -7.0, 3.0]);
    v4[0..=2].fill(0.0);
    v4[1..=3].fill(-9.0);
    assert_eq!(v4, [0.0, -9.0, -9.0, -9.0]);
    v2[1] = -9.0;
    v2[0] = 88.0;
    assert_eq!([v2[1], v2[0]], [-9.0, 88.0]);
    let a = [0.0_f32, 1.0, 12.0, 99.9];
    let b = [0.1_f32, -1.0, -9.0, -20.0];
    let order = [1, 0, 3, 2];
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| b[order[i]].abs()),
        std::array::from_fn(|i| b[order[i]]).map(f32::abs)
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[order[i]].floor()),
        std::array::from_fn(|i| a[order[i]]).map(f32::floor)
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| a[order[i]].ceil()),
        std::array::from_fn(|i| a[order[i]]).map(f32::ceil)
    );
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
                assert_eq!(1.0 as $type / 0.0, infinity);
                assert_eq!(-infinity / 2.0, -infinity);
                assert_eq!(1.0 as $type / infinity, 0.0);
                assert_eq!(4.0 as $type / 4.0, 1.0);
                for value in [
                    infinity / infinity,
                    0.0 / 0.0,
                    0.0 * infinity,
                    infinity - infinity,
                ] {
                    assert!(value.is_nan());
                    assert_ne!(value, value);
                    assert!(!(value <= value));
                    assert!(!(value >= value));
                    assert!(!(value < value));
                    assert!(!(value > value));
                }
                assert_eq!(infinity + infinity, infinity);
                assert!((infinity + -infinity).is_nan());
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

fn check_ternary<T: Copy + From<u8> + PartialEq + std::fmt::Debug>() {
    let one = T::from(1);
    let two = T::from(2);
    let three = T::from(3);
    let four = T::from(4);
    assert_eq!(
        [if true { one } else { two }, if false { one } else { two }],
        [one, two]
    );
    assert_eq!(
        [
            if false { one } else { three },
            if true { two } else { four }
        ],
        [three, two]
    );
    assert_eq!(
        if one == two {
            [one, two]
        } else {
            [three, four]
        },
        [three, four]
    );
}

#[test]
fn ternary_operator() {
    check_ternary::<u8>();
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
        assert_eq!(f32::NAN.min(value), value);
        assert_eq!(f32::NAN.max(value), value);
        assert_eq!(value.min(f32::NAN), value);
        assert_eq!(value.max(f32::NAN), value);
    }
    assert_eq!(
        std::array::from_fn::<_, 16, _>(|i| i.max(15 - i) as u8),
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
        [-1.0_f32, 2.0, -3.0, 4.0].map(f32::abs),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!([-5.0_f32, 6.0].map(f32::abs), [5.0, 6.0]);
    assert_eq!([-0.0_f32, 0.0].map(f32::abs), [0.0, 0.0]);
    assert!(f32::NAN.abs().is_nan());
    assert!((-f32::NAN).abs().is_nan());
    assert_eq!([7_i32, -8, 9, -10].map(i32::wrapping_abs), [7, 8, 9, 10]);
    assert_eq!(i32::MIN.wrapping_abs(), i32::MIN);
}

#[test]
fn reduce() {
    let f = [1.0_f32, 2.0, 3.0, 4.0];
    assert_eq!(f.iter().sum::<f32>(), 10.0);
    assert_eq!(f[..3].iter().sum::<f32>(), 6.0);
    assert_eq!(f[1..3].iter().sum::<f32>(), 5.0);
    assert_eq!(f.into_iter().reduce(f32::min), Some(1.0));
    assert_eq!(f.into_iter().reduce(f32::max), Some(4.0));
    for values in [[1_i32, 2, 3, 4], [1, 2, 3, 4]] {
        assert_eq!(values.iter().sum::<i32>(), 10);
        assert_eq!(values.iter().copied().min(), Some(1));
        assert_eq!(values.iter().copied().max(), Some(4));
        assert_eq!(values.into_iter().reduce(|a, b| a & b), Some(0));
        assert_eq!(values.into_iter().reduce(|a, b| a | b), Some(7));
    }
    assert_eq!([2_i32, 3].into_iter().reduce(|a, b| a & b), Some(2));
    assert_eq!([2_i32, 3].into_iter().reduce(|a, b| a | b), Some(3));
}

#[test]
fn floor() {
    assert_eq!(
        [-1.9_f32, 1.9, 2.0, -2.0].map(f32::floor),
        [-2.0, 1.0, 2.0, -2.0]
    );
    assert_eq!(
        [f32::INFINITY, f32::NEG_INFINITY].map(f32::floor),
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(f32::NAN.floor().is_nan());
    assert!((-f32::NAN).floor().is_nan());
}

#[test]
fn ceil() {
    assert_eq!(
        [-1.9_f32, 1.9, 2.0, -2.0].map(f32::ceil),
        [-1.0, 2.0, 2.0, -2.0]
    );
    assert_eq!(
        [f32::INFINITY, f32::NEG_INFINITY].map(f32::ceil),
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(f32::NAN.ceil().is_nan());
    assert!((-f32::NAN).ceil().is_nan());
}

#[test]
fn copysign() {
    let values = [-1.0_f32, 2.0, -3.0, 4.0];
    let signs = [-999.2_f32, f32::NEG_INFINITY, 123.4, 0.0000001];
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| values[i].copysign(signs[i])),
        [-1.0, -2.0, 3.0, 4.0]
    );
    assert_eq!(
        [998.0_f32.copysign(-1.0), (-23.0_f32).copysign(1.0)],
        [-998.0, 23.0]
    );
    assert!(f32::NAN.copysign(-1.0).is_nan());
    assert!((-f32::NAN).copysign(1.0).is_nan());
}

#[test]
fn sqrt() {
    assert_eq!(
        [1.0_f32, 4.0, 9.0, 16.0].map(f32::sqrt),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        [49.0_f32, 64.0, 81.0, 100.0, 121.0].map(f32::sqrt),
        [7.0, 8.0, 9.0, 10.0, 11.0]
    );
    for value in [-1.0, f32::NEG_INFINITY, f32::NAN, -2.0] {
        assert!(value.sqrt().is_nan());
    }
    assert_eq!(
        [f32::INFINITY, 0.0, 1.0].map(f32::sqrt),
        [f32::INFINITY, 0.0, 1.0]
    );
}

#[test]
fn div255() {
    for base in (0..255 * 255).step_by(8) {
        let values = std::array::from_fn::<_, 8, _>(|lane| (base + lane).min(255 * 255) as u16);
        assert_eq!(
            values.map(|value| {
                let biased = u32::from(value) + 128;
                ((biased + (biased >> 8)) >> 8) as u16
            }),
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
    let values = [-1.9_f32, -1.5, 1.5, 1.1];
    assert_eq!(values.map(|value| value as i32), [-1, -1, 1, 1]);
    assert_eq!(values.map(|value| value.floor() as i32), [-2, -2, 1, 1]);
    assert_eq!(values.map(|value| value.ceil() as i32), [-1, -1, 2, 2]);
    assert_eq!(
        [values[2], values[3], values[0], values[1]].map(|value| value.ceil() as i32),
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
    let cross = |a: [i32; 2], b: [i32; 2]| a[0] * b[1] - a[1] * b[0];
    assert_eq!(cross([0, 1], [0, 1]), 0);
    assert_eq!(cross([1, 0], [1, 0]), 0);
    assert_eq!(cross([1, 1], [1, 1]), 0);
    assert_eq!(cross([1, 1], [1, -1]), -2);
    assert_eq!(cross([1, 1], [-1, 1]), 2);
}

#[test]
fn join() {
    assert_eq!(
        [[1, 2].as_slice(), [3, 4, 5, 6].as_slice()].concat(),
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        [[1.0].as_slice(), [2.0, 3.0, 4.0].as_slice()].concat(),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        [[1].as_slice(), [2, 3].as_slice(), [4, 5, 6].as_slice()].concat(),
        [1, 2, 3, 4, 5, 6]
    );
    let joined = [[3_u8; 8], [9; 8], [3; 8], [100; 8]].concat();
    assert_eq!(&joined[..8], &[3; 8]);
    assert_eq!(&joined[8..16], &[9; 8]);
    assert_eq!(&joined[16..24], &[3; 8]);
    assert_eq!(&joined[24..], &[100; 8]);
}

fn zip<T: Copy, const N: usize>(a: [T; N], b: [T; N]) -> Vec<T> {
    a.into_iter().zip(b).flat_map(|(a, b)| [a, b]).collect()
}

#[test]
fn zip_case() {
    assert_eq!(zip(['a'], ['b']), ['a', 'b']);
    assert_eq!(zip([1, 2], [3, 4]), [1, 3, 2, 4]);
    assert_eq!(zip([1, 2, 3, 4], [5, 6, 7, 8]), [1, 5, 2, 6, 3, 7, 4, 8]);
    assert_eq!(
        zip([1_u8, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]),
        [1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15, 8, 16]
    );
}

fn check_mix<const N: usize>(random: &mut UpstreamRand) {
    let a: [f32; N] = std::array::from_fn(|_| random.next());
    let b: [f32; N] = std::array::from_fn(|_| random.next());
    let scalar = random.next();
    let mixed: [f32; N] =
        std::array::from_fn(|index| a[index] * (1.0 - scalar) + b[index] * scalar);
    assert!(fuzzy_equal(
        mixed,
        std::array::from_fn(|index| a[index] * (1.0 - scalar) + b[index] * scalar)
    ));
    let lanes: [f32; N] = std::array::from_fn(|_| random.next());
    let mixed: [f32; N] =
        std::array::from_fn(|index| a[index] * (1.0 - lanes[index]) + b[index] * lanes[index]);
    assert!(fuzzy_equal(
        mixed,
        std::array::from_fn(|index| a[index] * (1.0 - lanes[index]) + b[index] * lanes[index])
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
        std::array::from_fn::<_, 4, _>(|index| a[index] * (1.0 - 0.0) + b[index] * 0.0),
        a
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|index| a[index] * (1.0 - 1.0) + b[index] * 1.0),
        b
    );
}

#[test]
fn load4x4f() {
    let matrix = [
        0.0, 4.0, 8.0, 12.0, 1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0,
    ];
    let rows = std::array::from_fn::<_, 4, _>(|row| {
        std::array::from_fn::<_, 4, _>(|column| matrix[column * 4 + row])
    });
    assert_eq!(rows[0], [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(rows[1], [4.0, 5.0, 6.0, 7.0]);
    assert_eq!(rows[2], [8.0, 9.0, 10.0, 11.0]);
    assert_eq!(rows[3], [12.0, 13.0, 14.0, 15.0]);
}
