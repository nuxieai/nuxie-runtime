//! One-for-one ports of the eight cases in pinned
//! `tests/unit_tests/runtime/wangs_formula_test.cpp`.
//!
//! Rust currently has only a private cubic subdivision count inside the path
//! implementation. It has no counterpart for the upstream public quadratic,
//! conic, transform, log2, or worst-case Wang owner, so the owner-dependent
//! cases are executable expected-red tests. The reference-only tolerance case
//! remains green.

use std::ops::{Add, Mul, Sub};

const PRECISION: f32 = 4.0;
const EPSILON: f32 = 1.0 / 4096.0;
const TESSELLATION_TOLERANCE: f32 = 1.0 / 128.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct V {
    x: f32,
    y: f32,
}

impl V {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    fn dot(a: Self, b: Self) -> f32 {
        a.x.mul_add(b.x, a.y * b.y)
    }
}

impl Add for V {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for V {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for V {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Clone, Copy, Debug)]
struct M([f32; 4]);

impl Default for M {
    fn default() -> Self {
        Self([1.0, 0.0, 0.0, 1.0])
    }
}

impl M {
    fn map(self, point: V) -> V {
        V::new(
            self.0[0].mul_add(point.x, self.0[2] * point.y),
            self.0[1].mul_add(point.x, self.0[3] * point.y),
        )
    }
}

const SERP: [V; 4] = [
    V::new(285.625, 499.687),
    V::new(411.625, 808.188),
    V::new(1064.62, 135.688),
    V::new(1042.63, 585.187),
];
const LOOP: [V; 4] = [
    V::new(635.625, 614.687),
    V::new(171.625, 236.188),
    V::new(1064.62, 135.688),
    V::new(516.625, 570.187),
];
const QUAD: [V; 3] = [
    V::new(460.625, 557.187),
    V::new(707.121, 209.688),
    V::new(779.628, 577.687),
];

fn fuzzy_equal(a: f32, b: f32, tolerance: f32) -> bool {
    assert!(tolerance >= 0.0);
    (a - b).abs() <= tolerance
}

fn quadratic_reference(precision: f32, points: &[V; 3]) -> f32 {
    let k = (2.0 * 1.0) / 8.0 * precision;
    (k * (points[0] - points[1] * 2.0 + points[2]).length()).sqrt()
}

fn cubic_reference(precision: f32, points: &[V; 4]) -> f32 {
    let k = (3.0 * 2.0) / 8.0 * precision;
    (k * (points[0] - points[1] * 2.0 + points[2])
        .length()
        .max((points[1] - points[2] * 2.0 + points[3]).length()))
    .sqrt()
}

fn conic_reference(precision: f32, points: &[V; 3], weight: f32) -> f32 {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let center = V::new(0.5 * (min_x + max_x), 0.5 * (min_y + max_y));
    let translated = [points[0] - center, points[1] - center, points[2] - center];
    let max_len = translated
        .iter()
        .map(|point| point.length())
        .fold(0.0, f32::max);
    assert!(max_len > 0.0);
    let epsilon = 1.0 / precision;
    let numerator = 4.0 * weight.min(1.0) * epsilon;
    let denominator = (translated[2] - translated[1] * (2.0 * weight) + translated[0]).length()
        + (max_len - epsilon).max(0.0) * (2.0 - 2.0 * weight).abs();
    let delta = (numerator / denominator).sqrt();
    assert!(delta > 0.0);
    1.0 / delta
}

fn mix(a: V, b: V, t: f32) -> V {
    a * (1.0 - t) + b * t
}

fn chop_quad_at(source: &[V; 3], t: f32) -> [V; 5] {
    assert!(t > 0.0 && t < 1.0);
    let p01 = mix(source[0], source[1], t);
    let p12 = mix(source[1], source[2], t);
    [source[0], p01, mix(p01, p12, t), p12, source[2]]
}

fn eval_quad_at(source: &[V; 3], t: f32) -> V {
    assert!(t > 0.0 && t < 1.0);
    mix(
        mix(source[0], source[1], t),
        mix(source[1], source[2], t),
        t,
    )
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
        ((self.0 >> 33) as u32) as f32 / 0x7fff_ffff_u32 as f32
    }

    fn range(&mut self, minimum: f32, maximum: f32) -> f32 {
        minimum + self.next() * (maximum - minimum)
    }
}

fn random_beziers<const N: usize>(max_exponent: i32, mut visit: impl FnMut([V; N])) {
    let mut random = UpstreamRand::seeded();
    for exponent in -10..=max_exponent {
        let mut points = [V::default(); N];
        for point in &mut points {
            *point = V::new(
                (1.0 + random.next()) * 2.0_f32.powi(exponent),
                (1.0 + random.next()) * 2.0_f32.powi(exponent),
            );
        }
        visit(points);
    }
}

fn random_matrices(mut visit: impl FnMut(M)) {
    let mut random = UpstreamRand::seeded();
    visit(M::default());
    for i in -10..=30 {
        for j in -10..=30 {
            visit(M([
                (1.0 + random.next()) * 2.0_f32.powi(i),
                0.0,
                0.0,
                (1.0 + random.next()) * 2.0_f32.powi(j),
            ]));
            visit(M([
                (1.0 + random.next()) * 2.0_f32.powi(i),
                (1.0 + random.next()) * 2.0_f32.powi((j + i) / 2),
                (1.0 + random.next()) * 2.0_f32.powi((j + i) / 2),
                (1.0 + random.next()) * 2.0_f32.powi(j),
            ]));
        }
    }
}

fn missing_owner<T>() -> T {
    panic!("Rust has no standalone rive::wangs_formula owner")
}

fn quadratic(_points: &[V; 3], _precision: f32) -> f32 {
    missing_owner()
}
fn quadratic_log2(_points: &[V; 3], _precision: f32, _transform: Option<M>) -> i32 {
    missing_owner()
}
fn cubic(_points: &[V; 4], _precision: f32) -> f32 {
    missing_owner()
}
fn cubic_log2(_points: &[V; 4], _precision: f32, _transform: Option<M>) -> i32 {
    missing_owner()
}
fn worst_case_cubic(_width: f32, _height: f32, _precision: f32) -> f32 {
    missing_owner()
}
fn worst_case_cubic_log2(_width: f32, _height: f32, _precision: f32) -> i32 {
    missing_owner()
}
fn worst_case_cubic_pow4(_width: f32, _height: f32, _precision: f32) -> f32 {
    missing_owner()
}
fn conic(_precision: f32, _points: &[V; 3], _weight: f32, _transform: Option<M>) -> f32 {
    missing_owner()
}

fn setup_term(seed: i32, term: f32, cubic_curve: bool) -> [V; 4] {
    let mut seed = seed;
    let mut term_2d = if seed & 1 != 0 {
        V::new(term, 0.0)
    } else {
        V::new(0.5, 3.0_f32.sqrt() / 2.0) * term
    };
    seed >>= 1;
    if seed & 1 != 0 {
        term_2d.x = -term_2d.x;
    }
    seed >>= 1;
    if seed & 1 != 0 {
        std::mem::swap(&mut term_2d.x, &mut term_2d.y);
    }
    seed >>= 1;
    let mut points = [V::default(); 4];
    if cubic_curve {
        match seed % 4 {
            0 => {
                points[0] = term_2d;
                points[3] = term_2d * 0.75;
            }
            1 | 2 => points[1] = term_2d * -0.5,
            3 => {
                points[3] = term_2d;
                points[0] = term_2d * 0.75;
            }
            _ => unreachable!(),
        }
    } else {
        match seed % 3 {
            0 => points[0] = term_2d,
            1 => points[1] = term_2d * -0.5,
            2 => points[2] = term_2d,
            _ => unreachable!(),
        }
    }
    points
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_log2() {
    for level in 0..30 {
        let epsilon = EPSILON * 2.0_f32.powi(level * 2);
        let cubic_k = (3.0 * 2.0) / (8.0 * (1.0 / PRECISION));
        let cubic_x = 2.0_f32.powi(level * 2) / cubic_k;
        for (term, expected) in [(cubic_x - epsilon, level), (cubic_x + epsilon, level + 1)] {
            let points = setup_term(level << 1, term, true);
            let reference = cubic_reference(PRECISION, &points);
            assert_eq!(reference.log2().ceil() as i32, expected);
            let value = cubic(&points, PRECISION);
            assert!(fuzzy_equal(value / reference, 1.0, TESSELLATION_TOLERANCE));
            assert_eq!(cubic_log2(&points, PRECISION, None), expected);
        }

        let quadratic_k = 2.0 / (8.0 * (1.0 / PRECISION));
        let quadratic_x = 2.0_f32.powi(level * 2) / quadratic_k;
        for (term, expected) in [
            (quadratic_x - epsilon, level),
            (quadratic_x + epsilon, level + 1),
        ] {
            let points = setup_term(level << 1, term, false);
            let points = [points[0], points[1], points[2]];
            let reference = quadratic_reference(PRECISION, &points);
            assert_eq!(reference.log2().ceil() as i32, expected);
            let value = quadratic(&points, PRECISION);
            assert!(fuzzy_equal(value / reference, 1.0, TESSELLATION_TOLERANCE));
            assert_eq!(quadratic_log2(&points, PRECISION, None), expected);
        }
    }

    let check_cubic = |points: &[V; 4]| {
        let reference = cubic_reference(PRECISION, points).max(1.0);
        assert_eq!(
            reference.log2().ceil() as i32,
            cubic_log2(points, PRECISION, None)
        );
        assert!(fuzzy_equal(
            cubic(points, PRECISION).max(1.0) / reference,
            1.0,
            TESSELLATION_TOLERANCE
        ));
    };
    let check_quad = |points: &[V; 3]| {
        let reference = quadratic_reference(PRECISION, points).max(1.0);
        assert_eq!(
            reference.log2().ceil() as i32,
            quadratic_log2(points, PRECISION, None)
        );
        assert!(fuzzy_equal(
            quadratic(points, PRECISION).max(1.0) / reference,
            1.0,
            TESSELLATION_TOLERANCE
        ));
    };
    random_matrices(|matrix| {
        check_cubic(&SERP.map(|point| matrix.map(point)));
        check_cubic(&LOOP.map(|point| matrix.map(point)));
        check_quad(&QUAD.map(|point| matrix.map(point)));
    });
    random_beziers::<4>(30, |points| check_cubic(&points));
    random_beziers::<3>(30, |points| check_quad(&points));
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_vector_xforms() {
    random_matrices(|matrix| {
        for points in [SERP, LOOP] {
            let transformed = points.map(|point| matrix.map(point));
            assert_eq!(
                cubic_log2(&transformed, PRECISION, None),
                cubic_log2(&points, PRECISION, Some(matrix))
            );
        }
        let transformed = QUAD.map(|point| matrix.map(point));
        assert_eq!(
            quadratic_log2(&transformed, PRECISION, None),
            quadratic_log2(&QUAD, PRECISION, Some(matrix))
        );
        random_beziers::<4>(30, |points| {
            let transformed = points.map(|point| matrix.map(point));
            assert_eq!(
                cubic_log2(&transformed, PRECISION, None),
                cubic_log2(&points, PRECISION, Some(matrix))
            );
        });
        random_beziers::<3>(30, |points| {
            let transformed = points.map(|point| matrix.map(point));
            assert_eq!(
                quadratic_log2(&transformed, PRECISION, None),
                quadratic_log2(&points, PRECISION, Some(matrix))
            );
        });
    });
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_worst_case_cubic() {
    for points in [
        [
            V::new(0.0, 0.0),
            V::new(100.0, 100.0),
            V::default(),
            V::default(),
        ],
        [
            V::new(100.0, 100.0),
            V::new(100.0, 100.0),
            V::new(200.0, 200.0),
            V::new(100.0, 100.0),
        ],
    ] {
        assert_eq!(
            worst_case_cubic(100.0, 100.0, PRECISION),
            cubic_reference(PRECISION, &points)
        );
        assert_eq!(
            worst_case_cubic_log2(100.0, 100.0, PRECISION),
            cubic_log2(&points, PRECISION, None)
        );
    }
    for _ in 0..100 {
        random_beziers::<4>(30, |points| {
            let minimum = points.iter().fold(points[0], |value, point| {
                V::new(value.x.min(point.x), value.y.min(point.y))
            });
            let maximum = points.iter().fold(points[0], |value, point| {
                V::new(value.x.max(point.x), value.y.max(point.y))
            });
            let size = maximum - minimum;
            let worst = worst_case_cubic(size.x, size.y, PRECISION);
            let worst_log2 = worst_case_cubic_log2(size.x, size.y, PRECISION);
            assert!(worst >= cubic_reference(PRECISION, &points));
            assert_eq!(worst.max(1.0).log2().ceil() as i32, worst_log2);
        });
    }
    assert_eq!(
        worst_case_cubic_pow4(f32::INFINITY, f32::INFINITY, PRECISION),
        f32::INFINITY
    );
    assert_eq!(
        worst_case_cubic(f32::INFINITY, f32::INFINITY, PRECISION),
        f32::INFINITY
    );
}

#[test]
fn wangs_formula_quad_within_tol() {
    random_beziers::<3>(15, |points| {
        let segments = quadratic_reference(PRECISION, &points).ceil() as usize;
        let delta = 1.0 / segments as f32;
        for segment in 0..segments {
            let minimum = segment as f32 * delta;
            let maximum = (segment + 1) as f32 * delta;
            let section = if minimum == 0.0 {
                if maximum == 1.0 {
                    points
                } else {
                    let chopped = chop_quad_at(&points, maximum);
                    [chopped[0], chopped[1], chopped[2]]
                }
            } else {
                let first = chop_quad_at(&points, minimum);
                if maximum == 1.0 {
                    [first[2], first[3], first[4]]
                } else {
                    let tail = [first[2], first[3], first[4]];
                    let second = chop_quad_at(&tail, (maximum - minimum) / (1.0 - minimum));
                    [second[0], second[1], second[2]]
                }
            };
            let point = eval_quad_at(&section, 0.5);
            let normal = V::new(section[2].y - section[0].y, section[0].x - section[2].x);
            let distance = V::dot(point - section[0], normal).abs() / normal.length();
            assert!(distance <= (1.0 / PRECISION) + 1e-2);
        }
    });
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_rational_quad_reduces() {
    for _ in 0..100 {
        random_beziers::<3>(30, |points| {
            let rational = conic(PRECISION, &points, 1.0, None);
            let integral = quadratic_reference(PRECISION, &points);
            assert!(fuzzy_equal(rational, integral, TESSELLATION_TOLERANCE));
        });
    }
}

fn eval_conic(points: &[V; 3], weight: f64, t: f64) -> [f64; 2] {
    let eval = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        [(a[0] * t + b[0]) * t + c[0], (a[1] * t + b[1]) * t + c[1]]
    };
    let p0 = [points[0].x as f64, points[0].y as f64];
    let p1w = [points[1].x as f64 * weight, points[1].y as f64 * weight];
    let p2 = [points[2].x as f64, points[2].y as f64];
    let numerator = eval(
        [p2[0] - 2.0 * p1w[0] + p0[0], p2[1] - 2.0 * p1w[1] + p0[1]],
        [2.0 * (p1w[0] - p0[0]), 2.0 * (p1w[1] - p0[1])],
        p0,
    );
    let denominator = eval(
        [-2.0 * (weight - 1.0); 2],
        [2.0 * (weight - 1.0); 2],
        [1.0; 2],
    );
    [numerator[0] / denominator[0], numerator[1] / denominator[1]]
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_conic_within_tol() {
    let mut random = UpstreamRand::seeded();
    for exponent in -10..=10 {
        let weight = (1.0 + random.next()) * 2.0_f32.powi(exponent);
        random_beziers::<3>(24, |points| {
            let segments = conic(PRECISION, &points, weight, None).ceil() as usize;
            let delta = 1.0 / segments as f32;
            for segment in 0..segments {
                let minimum = segment as f32 * delta;
                let maximum = (segment + 1) as f32 * delta;
                let middle = 0.5 * (minimum + maximum);
                let p0 = eval_conic(&points, weight as f64, minimum as f64);
                let p1 = eval_conic(&points, weight as f64, middle as f64);
                let p2 = eval_conic(&points, weight as f64, maximum as f64);
                let normal = [p2[1] - p0[1], p0[0] - p2[0]];
                let length = normal[0].hypot(normal[1]);
                assert_ne!(length, 0.0);
                let distance =
                    ((p1[0] - p0[0]) * normal[0] + (p1[1] - p0[1]) * normal[1]).abs() / length;
                assert!(distance <= (1.0 / PRECISION as f64) + EPSILON as f64);
                assert!(distance <= (1.0 / PRECISION as f64) + EPSILON as f64);
            }
        });
    }
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_conic_matches_reference() {
    let mut random = UpstreamRand::seeded();
    for exponent in -10..=10 {
        let weight = (1.0 + random.next()) * 2.0_f32.powi(exponent);
        random_beziers::<3>(30, |points| {
            let reference = conic_reference(PRECISION, &points, weight);
            let segments = conic(PRECISION, &points, weight, None);
            assert!(fuzzy_equal(
                reference,
                segments,
                reference * (1.0 / 1_048_576.0)
            ));
        });
    }
}

#[test]
#[ignore = "expected-red: Rust has no standalone rive::wangs_formula owner"]
fn wangs_formula_conic_vector_xforms() {
    let mut random = UpstreamRand::seeded();
    for exponent in -10..=10 {
        let weight = (1.0 + random.next()) * 2.0_f32.powi(exponent);
        random_beziers::<3>(30, |points| {
            for matrix in [
                M::default(),
                M([
                    random.range(-10.0, 10.0),
                    0.0,
                    0.0,
                    random.range(-10.0, 10.0),
                ]),
                M([
                    random.range(-10.0, 10.0),
                    random.range(-10.0, 10.0),
                    random.range(-10.0, 10.0),
                    random.range(-10.0, 10.0),
                ]),
            ] {
                let transformed = points.map(|point| matrix.map(point));
                let expected = conic(PRECISION, &transformed, weight, None);
                let actual = conic(PRECISION, &points, weight, Some(matrix));
                assert!((actual - expected).abs() <= 1e-4);
            }
        });
    }
}
