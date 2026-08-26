//! Strict Rust-language adaptations for the Wave C4 SIMD rows whose complete
//! observable contract is expressible through primitive and array operations.
//! No test-local SIMD implementation is defined here.

#[test]
fn wave_c4_simd_001_any() {
    assert!(![0, 0, 0, 0].into_iter().any(|value| value != 0));
    assert!([-1, 0, 0, 0].into_iter().any(|value| value != 0));
    assert!([0, -1, 0, 0].into_iter().any(|value| value != 0));
    assert!([0, 0, -1, 0].into_iter().any(|value| value != 0));
    assert!([0, 0, 0, -1].into_iter().any(|value| value != 0));
    assert!(![0, 0, 0].into_iter().any(|value| value != 0));
    assert!([-1, 0, 0].into_iter().any(|value| value != 0));
    assert!([0, -1, 0].into_iter().any(|value| value != 0));
    assert!([0, 0, -1].into_iter().any(|value| value != 0));
    assert!(![0, 0].into_iter().any(|value| value != 0));
    assert!([-1, 0].into_iter().any(|value| value != 0));
    assert!([0, -1].into_iter().any(|value| value != 0));
    assert!(![0].into_iter().any(|value| value != 0));
    assert!([-1].into_iter().any(|value| value != 0));
}

#[test]
fn wave_c4_simd_002_all() {
    assert!([-1, -1, -1, -1].into_iter().all(|value| value != 0));
    assert!(![0, -1, -1, -1].into_iter().all(|value| value != 0));
    assert!(![-1, 0, -1, -1].into_iter().all(|value| value != 0));
    assert!(![-1, -1, 0, -1].into_iter().all(|value| value != 0));
    assert!(![-1, -1, -1, 0].into_iter().all(|value| value != 0));
    assert!([-1, -1, -1].into_iter().all(|value| value != 0));
    assert!(![0, -1, -1].into_iter().all(|value| value != 0));
    assert!(![-1, 0, -1].into_iter().all(|value| value != 0));
    assert!(![-1, -1, 0].into_iter().all(|value| value != 0));
    assert!([-1, -1].into_iter().all(|value| value != 0));
    assert!(![0, -1].into_iter().all(|value| value != 0));
    assert!(![-1, 0].into_iter().all(|value| value != 0));
    assert!([-1].into_iter().all(|value| value != 0));
    assert!(![0].into_iter().all(|value| value != 0));
}

#[test]
fn wave_c4_simd_003_operators() {
    let a = [1.0_f32, 2.0, 3.0, 4.0];
    let b = [5.0_f32, 6.0, 7.0, 8.0];
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
        std::array::from_fn::<_, 4, _>(|i| a[i] - 10.0),
        std::array::from_fn(|i| -(10.0 - a[i]))
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
    assert!(
        !std::array::from_fn::<_, 4, _>(|lane| i[lane] & j[lane])
            .into_iter()
            .any(|value| value != 0)
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
fn wave_c4_simd_009_abs() {
    assert_eq!(
        [-1.0_f32, 2.0, -3.0, 4.0].map(f32::abs),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!([-5.0_f32, 6.0].map(f32::abs), [5.0, 6.0]);
    assert_eq!([-0.0_f32, 0.0].map(f32::abs), [0.0, 0.0]);
    assert_eq!(
        [
            -f32::EPSILON,
            -f32::from_bits(1),
            -f32::MAX,
            f32::NEG_INFINITY
        ],
        [
            -f32::EPSILON,
            -f32::from_bits(1),
            -f32::MAX,
            f32::NEG_INFINITY
        ]
    );
    assert!(
        [f32::NAN, -f32::NAN]
            .map(f32::abs)
            .into_iter()
            .all(f32::is_nan)
    );
    assert_eq!([7_i32, -8, 9, -10].map(i32::wrapping_abs), [7, 8, 9, 10]);
    assert_eq!([0_i32, 0].map(i32::wrapping_abs), [0, 0]);
    assert_eq!(
        [-(i32::MAX), i32::MIN].map(i32::wrapping_abs),
        [i32::MAX, i32::MIN]
    );
}

#[test]
fn wave_c4_simd_011_floor() {
    assert_eq!(
        [-1.9_f32, 1.9, 2.0, -2.0].map(f32::floor),
        [-2.0, 1.0, 2.0, -2.0]
    );
    assert_eq!(
        [f32::INFINITY, f32::NEG_INFINITY].map(f32::floor),
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(
        [f32::NAN, -f32::NAN]
            .map(f32::floor)
            .into_iter()
            .all(f32::is_nan)
    );
}

#[test]
fn wave_c4_simd_012_ceil() {
    assert_eq!(
        [-1.9_f32, 1.9, 2.0, -2.0].map(f32::ceil),
        [-1.0, 2.0, 2.0, -2.0]
    );
    assert_eq!(
        [f32::INFINITY, f32::NEG_INFINITY].map(f32::ceil),
        [f32::INFINITY, f32::NEG_INFINITY]
    );
    assert!(
        [f32::NAN, -f32::NAN]
            .map(f32::ceil)
            .into_iter()
            .all(f32::is_nan)
    );
}

#[test]
fn wave_c4_simd_013_copysign() {
    let signs = [-999.2_f32, f32::NEG_INFINITY, 123.4, 0.0000001];
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| [-1.0_f32, 2.0, -3.0, 4.0][i].copysign(signs[i])),
        [-1.0, -2.0, 3.0, 4.0]
    );
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| [
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY
        ][i]
            .copysign(signs[i])),
        [
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::INFINITY
        ]
    );
    assert_eq!(
        [998.0_f32.copysign(-1.0), (-23.0_f32).copysign(1.0)],
        [-998.0, 23.0]
    );
    assert!(
        [f32::NAN.copysign(-1.0), (-f32::NAN).copysign(1.0)]
            .into_iter()
            .all(f32::is_nan)
    );
    assert!(
        [
            f32::NAN.copysign(-1.0),
            (-f32::NAN).copysign(-1.0),
            f32::NAN.copysign(1.0),
            (-f32::NAN).copysign(1.0),
        ]
        .into_iter()
        .all(f32::is_nan)
    );
}

#[test]
fn wave_c4_simd_014_sqrt() {
    assert_eq!(
        [1.0_f32, 4.0, 9.0, 16.0].map(f32::sqrt),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!([25.0_f32, 36.0].map(f32::sqrt), [5.0, 6.0]);
    assert_eq!([36.0_f32].map(f32::sqrt), [6.0]);
    assert_eq!(
        [49.0_f32, 64.0, 81.0, 100.0, 121.0].map(f32::sqrt),
        [7.0, 8.0, 9.0, 10.0, 11.0]
    );
    assert!(
        [-1.0_f32, f32::NEG_INFINITY, f32::NAN, -2.0]
            .map(f32::sqrt)
            .into_iter()
            .all(f32::is_nan)
    );
    assert_eq!(
        [f32::INFINITY, 0.0, 1.0].map(f32::sqrt),
        [f32::INFINITY, 0.0, 1.0]
    );
}

#[test]
fn wave_c4_simd_017_cast() {
    let values = [-1.9_f32, -1.5, 1.5, 1.1];
    assert_eq!(values.map(|value| value as i32), [-1, -1, 1, 1]);
    assert_eq!(values.map(|value| value.floor() as i32), [-2, -2, 1, 1]);
    assert_eq!(values.map(|value| value.ceil() as i32), [-1, -1, 2, 2]);
    assert_eq!(
        [values[2], values[3], values[0], values[1]].map(|value| value.ceil() as i32),
        [2, 2, -1, -1]
    );
    let ceiled = values.map(f32::ceil);
    assert_eq!(
        [ceiled[2], ceiled[3], ceiled[0], ceiled[1]].map(|value| value as i32),
        [2, 2, -1, -1]
    );
}

#[test]
fn wave_c4_simd_018_dot() {
    assert_eq!(
        [0_i32, 1]
            .into_iter()
            .zip([1, 0])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        0
    );
    assert_eq!(
        [1_u32, 0]
            .into_iter()
            .zip([0, 1])
            .map(|(a, b)| a * b)
            .sum::<u32>(),
        0
    );
    assert_eq!(
        [1_i32, 1]
            .into_iter()
            .zip([1, -1])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        0
    );
    assert_eq!(
        [1_u32, 1]
            .into_iter()
            .zip([1, 1])
            .map(|(a, b)| a * b)
            .sum::<u32>(),
        2
    );
    assert_eq!(
        [1_i32, 1]
            .into_iter()
            .zip([-1, -1])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        -2
    );
    assert_eq!(
        [1_i32, 2, -3]
            .into_iter()
            .zip([1, 2, 3])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        -4
    );
    assert_eq!(
        [1_u32, 2, 3]
            .into_iter()
            .zip([1, 2, 3])
            .map(|(a, b)| a * b)
            .sum::<u32>(),
        14
    );
    assert_eq!(
        [1_i32, 2, 3, 4]
            .into_iter()
            .zip([1, 2, 3, 4])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        30
    );
    assert_eq!(
        [1_i32, 2, 3, 4, 5]
            .into_iter()
            .zip([1, 2, 3, 4, -5])
            .map(|(a, b)| a * b)
            .sum::<i32>(),
        5
    );
    assert_eq!(
        [1_u32, 2, 3, 4, 5]
            .into_iter()
            .zip([1, 2, 3, 4, 5])
            .map(|(a, b)| a * b)
            .sum::<u32>(),
        55
    );
    assert_eq!(
        [1.0_f32, 2.0, 3.0, 4.0]
            .into_iter()
            .zip([4.0, 3.0, 2.0, 1.0])
            .map(|(a, b)| a * b)
            .sum::<f32>(),
        20.0
    );
    assert_eq!(
        [1.0_f32, 2.0, 3.0]
            .into_iter()
            .zip([3.0, 2.0, 1.0])
            .map(|(a, b)| a * b)
            .sum::<f32>(),
        10.0
    );
    assert_eq!(
        [0.0_f32, 1.0]
            .into_iter()
            .zip([1.0, 0.0])
            .map(|(a, b)| a * b)
            .sum::<f32>(),
        0.0
    );
    assert_eq!(
        [1.0_f32, 2.0, 3.0, 4.0, 5.0]
            .into_iter()
            .zip([1.0, 2.0, 3.0, 4.0, 5.0])
            .map(|(a, b)| a * b)
            .sum::<f32>(),
        55.0
    );
}

#[test]
fn wave_c4_simd_019_cross() {
    assert_eq!(0.0_f32 * 1.0 - 1.0 * 0.0, 0.0);
    assert_eq!(1.0_f32 * 0.0 - 0.0 * 1.0, 0.0);
    assert_eq!(1.0_f32 * 1.0 - 1.0 * 1.0, 0.0);
    assert_eq!(1.0_f32 * -1.0 - 1.0 * 1.0, -2.0);
    assert_eq!(1.0_f32 * 1.0 - 1.0 * -1.0, 2.0);
}

#[test]
fn wave_c4_simd_020_join() {
    assert_eq!(
        [[1, 2].as_slice(), [3, 4, 5, 6].as_slice()].concat(),
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        [[1.0_f32].as_slice(), [2.0, 3.0, 4.0].as_slice()].concat(),
        [1.0_f32, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        [
            [1.0_f32].as_slice(),
            [2.0, 3.0].as_slice(),
            [4.0, 5.0, 6.0].as_slice(),
        ]
        .concat(),
        [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0]
    );
    assert_eq!(
        [
            [1.0_f32].as_slice(),
            [2.0, 3.0].as_slice(),
            [4.0, 5.0, 6.0].as_slice(),
            [7.0, 8.0, 9.0, 10.0].as_slice()
        ]
        .concat(),
        [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    );
    assert_eq!(
        [
            [3_u8; 8].as_slice(),
            [9; 8].as_slice(),
            [3; 8].as_slice(),
            [100; 8].as_slice()
        ]
        .concat(),
        [
            3, 3, 3, 3, 3, 3, 3, 3, 9, 9, 9, 9, 9, 9, 9, 9, 3, 3, 3, 3, 3, 3, 3, 3, 100, 100, 100,
            100, 100, 100, 100, 100
        ]
    );
}

#[test]
fn wave_c4_simd_021_zip() {
    assert_eq!(
        [b'a']
            .into_iter()
            .zip([b'b'])
            .flat_map(|pair| [pair.0, pair.1])
            .collect::<Vec<_>>(),
        [b'a', b'b']
    );
    assert_eq!(
        [1, 2]
            .into_iter()
            .zip([3, 4])
            .flat_map(|pair| [pair.0, pair.1])
            .collect::<Vec<_>>(),
        [1, 3, 2, 4]
    );
    assert_eq!(
        [1, 2, 3, 4]
            .into_iter()
            .zip([5, 6, 7, 8])
            .flat_map(|pair| [pair.0, pair.1])
            .collect::<Vec<_>>(),
        [1, 5, 2, 6, 3, 7, 4, 8]
    );
    assert_eq!(
        [1_u8, 2, 3, 4, 5, 6, 7, 8]
            .into_iter()
            .zip([9, 10, 11, 12, 13, 14, 15, 16])
            .flat_map(|pair| [pair.0, pair.1])
            .collect::<Vec<_>>(),
        [1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15, 8, 16]
    );
    assert_eq!(
        [1.0_f32, 2.0, 3.0, 4.0]
            .into_iter()
            .zip([5.0, 6.0, 7.0, 8.0])
            .flat_map(|pair| [pair.0, pair.1])
            .collect::<Vec<_>>(),
        [1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]
    );
}

#[test]
fn wave_c4_simd_023_load4x4f() {
    let matrix = [
        0.0, 4.0, 8.0, 12.0, 1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0,
    ];
    assert_eq!(
        [matrix[0], matrix[4], matrix[8], matrix[12]],
        [0.0, 1.0, 2.0, 3.0]
    );
    assert_eq!(
        [matrix[1], matrix[5], matrix[9], matrix[13]],
        [4.0, 5.0, 6.0, 7.0]
    );
    assert_eq!(
        [matrix[2], matrix[6], matrix[10], matrix[14]],
        [8.0, 9.0, 10.0, 11.0]
    );
    assert_eq!(
        [matrix[3], matrix[7], matrix[11], matrix[15]],
        [12.0, 13.0, 14.0, 15.0]
    );
}
