//! Preserved SIMD assertions now exercise the translated production owner.
use nuxie_runtime::source::math::simd::{self, GVec};
fn g<T: Copy, const N: usize>(values: [T; N]) -> GVec<T, N> {
    GVec::from_array(values)
}
#[test]
fn wave_c4_simd_001_any() {
    assert!(!simd::any(g([0, 0, 0, 0])));
    assert!(simd::any(g([-1, 0, 0, 0])));
    assert!(simd::any(g([0, -1, 0, 0])));
    assert!(simd::any(g([0, 0, -1, 0])));
    assert!(simd::any(g([0, 0, 0, -1])));
    assert!(!simd::any(g([0, 0, 0])));
    assert!(simd::any(g([-1, 0, 0])));
    assert!(simd::any(g([0, -1, 0])));
    assert!(simd::any(g([0, 0, -1])));
    assert!(!simd::any(g([0, 0])));
    assert!(simd::any(g([-1, 0])));
    assert!(simd::any(g([0, -1])));
    assert!(!simd::any(g([0])));
    assert!(simd::any(g([-1])));
}
#[test]
fn wave_c4_simd_002_all() {
    assert!(simd::all(g([-1, -1, -1, -1])));
    assert!(!simd::all(g([0, -1, -1, -1])));
    assert!(!simd::all(g([-1, 0, -1, -1])));
    assert!(!simd::all(g([-1, -1, 0, -1])));
    assert!(!simd::all(g([-1, -1, -1, 0])));
    assert!(simd::all(g([-1, -1, -1])));
    assert!(!simd::all(g([0, -1, -1])));
    assert!(!simd::all(g([-1, 0, -1])));
    assert!(!simd::all(g([-1, -1, 0])));
    assert!(simd::all(g([-1, -1])));
    assert!(!simd::all(g([0, -1])));
    assert!(!simd::all(g([-1, 0])));
    assert!(simd::all(g([-1])));
    assert!(!simd::all(g([0])));
}
#[test]
fn wave_c4_simd_003_operators() {
    let a = g([1.0_f32, 2.0, 3.0, 4.0]);
    let b = g([5.0_f32, 6.0, 7.0, 8.0]);
    assert_eq!((a + b).data, [6.0, 8.0, 10.0, 12.0]);
    assert_eq!((a - b).data, [-4.0; 4]);
    assert_eq!((a * b).data, [5.0, 12.0, 21.0, 32.0]);
    assert_eq!(a / a, b / b);
    assert_eq!(a + 10.0, 10.0 + a);
    assert_eq!(a - 10.0, -(10.0 - a));
    assert_eq!(a * 2.0, 2.0 * a);
    assert_eq!(a / 0.5, 2.0 * a);
    let i = g([1_i32, 2, 4, 8]);
    let j = g([0_i32, 1, 3, 7]);
    assert!(!simd::any(i & j));
    assert_eq!((i | j).data, [1, 3, 7, 15]);
    assert_eq!(i | j, i ^ j);
    assert_eq!(!i + 1, -i);
}
#[test]
fn wave_c4_simd_009_abs() {
    assert_eq!(
        simd::abs(g([-1.0_f32, 2.0, -3.0, 4.0])).data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(simd::abs(g([-5.0_f32, 6.0])).data, [5.0, 6.0]);
    assert_eq!(simd::abs(g([-0.0_f32, 0.0])).data, [0.0, 0.0]);
    // This equality-only edge case is also written this way in the pin.
    assert_eq!(
        g([
            -f32::EPSILON,
            -f32::from_bits(1),
            -f32::MAX,
            f32::NEG_INFINITY
        ]),
        g([
            -f32::EPSILON,
            -f32::from_bits(1),
            -f32::MAX,
            f32::NEG_INFINITY
        ])
    );
    assert!(simd::all(simd::isnan(simd::abs(g([f32::NAN, -f32::NAN])))));
    assert_eq!(simd::abs(g([7_i32, -8, 9, -10])).data, [7, 8, 9, 10]);
    assert_eq!(simd::abs(g([0_i32, 0])).data, [0, 0]);
    assert_eq!(
        simd::abs(g([-i32::MAX, i32::MIN])).data,
        [i32::MAX, i32::MIN]
    );
}
#[test]
fn wave_c4_simd_011_floor() {
    assert_eq!(
        simd::floor(g([-1.9_f32, 1.9, 2.0, -2.0])).data,
        [-2.0, 1.0, 2.0, -2.0]
    );
    assert_eq!(
        simd::floor(g([f32::INFINITY, f32::NEG_INFINITY])).data,
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(simd::all(simd::isnan(simd::floor(g([
        f32::NAN,
        -f32::NAN
    ])))));
}
#[test]
fn wave_c4_simd_012_ceil() {
    assert_eq!(
        simd::ceil(g([-1.9_f32, 1.9, 2.0, -2.0])).data,
        [-1.0, 2.0, 2.0, -2.0]
    );
    assert_eq!(
        simd::ceil(g([f32::INFINITY, f32::NEG_INFINITY])).data,
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(simd::all(simd::isnan(simd::ceil(g([f32::NAN, -f32::NAN])))));
}
#[test]
fn wave_c4_simd_013_copysign() {
    let signs = g([-999.2_f32, f32::NEG_INFINITY, 123.4, 0.0000001]);
    assert_eq!(
        simd::copy_sign(g([-1.0_f32, 2.0, -3.0, 4.0]), signs).data,
        [-1.0, -2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::copy_sign(
            g([
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY
            ]),
            signs
        )
        .data,
        [
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::INFINITY
        ]
    );
    assert_eq!(
        simd::copy_sign(g([998.0_f32, -23.0]), g([-1.0, 1.0])).data,
        [-998.0, 23.0]
    );
    assert!(simd::all(simd::isnan(simd::copy_sign(
        g([f32::NAN, -f32::NAN]),
        g([-1.0, 1.0])
    ))));
    assert!(simd::all(simd::isnan(simd::copy_sign(
        g([f32::NAN, -f32::NAN, f32::NAN, -f32::NAN]),
        g([-1.0, -1.0, 1.0, 1.0])
    ))));
}
#[test]
fn wave_c4_simd_014_sqrt() {
    assert_eq!(
        simd::sqrt(g([1.0_f32, 4.0, 9.0, 16.0])).data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(simd::sqrt(g([25.0_f32, 36.0])).data, [5.0, 6.0]);
    assert_eq!(simd::sqrt(g([36.0_f32])).data, [6.0]);
    assert_eq!(
        simd::sqrt(g([49.0_f32, 64.0, 81.0, 100.0, 121.0])).data,
        [7.0, 8.0, 9.0, 10.0, 11.0]
    );
    assert!(simd::all(simd::isnan(simd::sqrt(g([
        -1.0_f32,
        f32::NEG_INFINITY,
        f32::NAN,
        -2.0
    ])))));
    assert_eq!(
        simd::sqrt(g([f32::INFINITY, 0.0, 1.0])).data,
        [f32::INFINITY, 0.0, 1.0]
    );
}
#[test]
fn wave_c4_simd_017_cast() {
    let values = g([-1.9_f32, -1.5, 1.5, 1.1]);
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
    assert_eq!(
        simd::cast::<i32, _, 4>(simd::ceil(values).zwxy()).data,
        [2, 2, -1, -1]
    );
}
#[test]
fn wave_c4_simd_018_dot() {
    assert_eq!(simd::dot(g([0_i32, 1]), g([1, 0])), 0);
    assert_eq!(simd::dot(g([1_u32, 0]), g([0, 1])), 0);
    assert_eq!(simd::dot(g([1_i32, 1]), g([1, -1])), 0);
    assert_eq!(simd::dot(g([1_u32, 1]), g([1, 1])), 2);
    assert_eq!(simd::dot(g([1_i32, 1]), g([-1, -1])), -2);
    assert_eq!(simd::dot(g([1_i32, 2, -3]), g([1, 2, 3])), -4);
    assert_eq!(simd::dot(g([1_u32, 2, 3]), g([1, 2, 3])), 14);
    assert_eq!(simd::dot(g([1_i32, 2, 3, 4]), g([1, 2, 3, 4])), 30);
    assert_eq!(simd::dot(g([1_i32, 2, 3, 4, 5]), g([1, 2, 3, 4, -5])), 5);
    assert_eq!(simd::dot(g([1_u32, 2, 3, 4, 5]), g([1, 2, 3, 4, 5])), 55);
    assert_eq!(
        simd::dot(g([1.0_f32, 2.0, 3.0, 4.0]), g([4.0, 3.0, 2.0, 1.0])),
        20.0
    );
    assert_eq!(simd::dot(g([1.0_f32, 2.0, 3.0]), g([3.0, 2.0, 1.0])), 10.0);
    assert_eq!(simd::dot(g([0.0_f32, 1.0]), g([1.0, 0.0])), 0.0);
    assert_eq!(
        simd::dot(
            g([1.0_f32, 2.0, 3.0, 4.0, 5.0]),
            g([1.0, 2.0, 3.0, 4.0, 5.0])
        ),
        55.0
    );
}
#[test]
fn wave_c4_simd_019_cross() {
    assert_eq!(simd::cross(g([0.0, 1.0]), g([0.0, 1.0])), 0.0);
    assert_eq!(simd::cross(g([1.0, 0.0]), g([1.0, 0.0])), 0.0);
    assert_eq!(simd::cross(g([1.0, 1.0]), g([1.0, 1.0])), 0.0);
    assert_eq!(simd::cross(g([1.0, 1.0]), g([1.0, -1.0])), -2.0);
    assert_eq!(simd::cross(g([1.0, 1.0]), g([-1.0, 1.0])), 2.0);
}
#[test]
fn wave_c4_simd_020_join() {
    assert_eq!(
        simd::join::<_, 2, 4, 6>(g([1, 2]), g([3, 4, 5, 6])).data,
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        simd::join::<_, 1, 3, 4>(g([1.0_f32]), g([2.0, 3.0, 4.0])).data,
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        simd::join3::<_, 1, 2, 3, 6>(g([1.0_f32]), g([2.0, 3.0]), g([4.0, 5.0, 6.0])).data,
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    );
    assert_eq!(
        simd::join4::<_, 1, 2, 3, 4, 10>(
            g([1.0_f32]),
            g([2.0, 3.0]),
            g([4.0, 5.0, 6.0]),
            g([7.0, 8.0, 9.0, 10.0])
        )
        .data,
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    );
    assert_eq!(
        simd::join4::<_, 8, 8, 8, 8, 32>(g([3_u8; 8]), g([9; 8]), g([3; 8]), g([100; 8])).data,
        [
            3, 3, 3, 3, 3, 3, 3, 3, 9, 9, 9, 9, 9, 9, 9, 9, 3, 3, 3, 3, 3, 3, 3, 3, 100, 100, 100,
            100, 100, 100, 100, 100
        ]
    );
}
#[test]
fn wave_c4_simd_021_zip() {
    assert_eq!(
        simd::zip::<_, 1, 2>(g([b'a']), g([b'b'])).data,
        [b'a', b'b']
    );
    assert_eq!(
        simd::zip::<_, 2, 4>(g([1, 2]), g([3, 4])).data,
        [1, 3, 2, 4]
    );
    assert_eq!(
        simd::zip::<_, 4, 8>(g([1, 2, 3, 4]), g([5, 6, 7, 8])).data,
        [1, 5, 2, 6, 3, 7, 4, 8]
    );
    assert_eq!(
        simd::zip::<_, 8, 16>(
            g([1_u8, 2, 3, 4, 5, 6, 7, 8]),
            g([9, 10, 11, 12, 13, 14, 15, 16])
        )
        .data,
        [1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15, 8, 16]
    );
    assert_eq!(
        simd::zip::<_, 4, 8>(g([1.0_f32, 2.0, 3.0, 4.0]), g([5.0, 6.0, 7.0, 8.0])).data,
        [1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]
    );
}
#[test]
fn wave_c4_simd_023_load4x4f() {
    let matrix = [
        0.0, 4.0, 8.0, 12.0, 1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0,
    ];
    let (a, b, c, d) = simd::load4x4f(&matrix);
    assert_eq!(a.data, [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(b.data, [4.0, 5.0, 6.0, 7.0]);
    assert_eq!(c.data, [8.0, 9.0, 10.0, 11.0]);
    assert_eq!(d.data, [12.0, 13.0, 14.0, 15.0]);
}
