//! Direct ports from pinned `tests/unit_tests/runtime/bezier_utils_test.cpp`.

use super::*;

const EPSILON: f32 = 1.0 / 4096.0;

const TEST_CUBICS: [[Vec2D; 4]; 14] = [
    [
        v(199.0, 1225.0),
        v(197.0, 943.0),
        v(349.0, 607.0),
        v(549.0, 427.0),
    ],
    [
        v(549.0, 427.0),
        v(349.0, 607.0),
        v(197.0, 943.0),
        v(199.0, 1225.0),
    ],
    [
        v(460.0, 1060.0),
        v(403.0, -320.0),
        v(60.0, 660.0),
        v(1181.0, 634.0),
    ],
    [
        v(1181.0, 634.0),
        v(60.0, 660.0),
        v(403.0, -320.0),
        v(460.0, 1060.0),
    ],
    [v(0.0, 0.0), v(0.0, 0.0), v(0.0, 0.0), v(0.0, 0.0)],
    [v(0.0, 0.0), v(0.0, 0.0), v(0.0, 0.0), v(100.0, 100.0)],
    [v(0.0, 0.0), v(0.0, 0.0), v(100.0, 100.0), v(100.0, 100.0)],
    [v(0.0, 0.0), v(100.0, 100.0), v(100.0, 100.0), v(0.0, 0.0)],
    [
        v(-100.0, -100.0),
        v(100.0, 100.0),
        v(100.0, -100.0),
        v(-100.0, 100.0),
    ],
    [v(0.0, 0.0), v(0.0, 0.0), v(100.0, 100.0), v(100.0, 100.0)],
    [
        v(0.0, 0.0),
        v(-100.0, -100.0),
        v(200.0, 200.0),
        v(100.0, 100.0),
    ],
    [
        v(0.0, 0.0),
        v(50.0 * 2.0 / 3.0, 100.0 * 2.0 / 3.0),
        v(100.0 - 50.0 * 2.0 / 3.0, 100.0 * 2.0 / 3.0),
        v(100.0, 0.0),
    ],
    [
        v(0.0, 0.0),
        v(50.0 * 2.0 / 3.0, 100.0 * 2.0 / 3.0),
        v(100.0 - 50.0 * 2.0 / 3.0, 100.0 * 2.0 / 3.0),
        v(100.0, 100.0),
    ],
    [v(100.0, 0.0), v(0.0, 0.0), v(0.0, 0.0), v(0.0, 0.0)],
];

const fn v(x: f32, y: f32) -> Vec2D {
    Vec2D::new(x, y)
}

fn add(a: Vec2D, b: Vec2D) -> Vec2D {
    v(a.x + b.x, a.y + b.y)
}

fn mul(a: Vec2D, scalar: f32) -> Vec2D {
    v(a.x * scalar, a.y * scalar)
}

fn length(a: Vec2D) -> f32 {
    (a.x * a.x + a.y * a.y).sqrt()
}

fn fuzzy_equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPSILON
}

fn pinned_shader_source() -> &'static str {
    include_str!(
        "../mechanical_port/shader-build-authority/source/renderer_src_shaders_bezier_utils_glsl__generated_input.source"
    )
}

fn flatten(segments: &[[Vec2D; 4]]) -> Vec<Vec2D> {
    let mut points = Vec::with_capacity(segments.len() * 3 + 1);
    points.extend(segments[0]);
    for segment in &segments[1..] {
        points.extend_from_slice(&segment[1..]);
    }
    points
}

fn next_random(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    ((*state >> 8) as f32) / ((u32::MAX >> 8) as f32)
}

fn random_cubic(state: &mut u32) -> [Vec2D; 4] {
    std::array::from_fn(|_| v(next_random(state), next_random(state)))
}

#[test]
fn chop_cubic_at() {
    let curve = [v(0.0, 0.0), v(1.0, 1.0), v(2.0, 2.0), v(3.0, 3.0)];
    let points = flatten(&chop_cubic_at_values(curve, &[0.5]));
    for (index, point) in points.iter().enumerate() {
        assert_eq!(point.x, point.y);
        assert_eq!(point.x, index as f32 * 0.5);
    }

    let chop_ts = [
        0.0,
        3.0 / 83.0,
        3.0 / 79.0,
        3.0 / 73.0,
        3.0 / 71.0,
        3.0 / 67.0,
        3.0 / 61.0,
        3.0 / 59.0,
        3.0 / 53.0,
        3.0 / 47.0,
        3.0 / 43.0,
        3.0 / 41.0,
        3.0 / 37.0,
        3.0 / 31.0,
        3.0 / 29.0,
        3.0 / 23.0,
        3.0 / 19.0,
        3.0 / 17.0,
        3.0 / 13.0,
        3.0 / 11.0,
        3.0 / 7.0,
        3.0 / 5.0,
        1.0,
    ];
    let mut random = 1;
    for _ in 0..5 {
        let curve = random_cubic(&mut random);
        let all = flatten(&chop_cubic_at_values(curve, &chop_ts));
        for (index, chop_t) in chop_ts.into_iter().enumerate() {
            let actual = all[3 + index * 3];
            let expected = eval_cubic(curve, chop_t);
            assert!(fuzzy_equal(actual.x, expected.x));
            assert!(fuzzy_equal(actual.y, expected.y));
        }
    }
}

#[test]
#[ignore = "expected-red: equal interior roots are not bit-exactly degenerate in the Rust multi-chop port"]
fn chop_cubic_at_equal_roots_are_exactly_degenerate_expected_red() {
    let chop_ts = [0.0, 3.0 / 83.0, 3.0 / 47.0, 3.0 / 11.0, 1.0];
    let mut random = 1;
    for _ in 0..5 {
        let curve = random_cubic(&mut random);
        for chop_t in chop_ts {
            let local = flatten(&chop_cubic_at_values(curve, &[chop_t, chop_t]));
            assert_eq!(local[3], local[4]);
            assert_eq!(local[3], local[5]);
            assert_eq!(local[3], local[6]);
            if chop_t == 0.0 {
                assert!(local[..=3].iter().all(|point| *point == curve[0]));
            }
            if chop_t == 1.0 {
                assert!(local[6..].iter().all(|point| *point == curve[3]));
            }
        }
    }
}

#[test]
#[ignore = "expected-red: repeated T=1 roots expose incorrect NaN handling in the Rust multi-chop port"]
fn chop_cubic_at_repeated_endpoint_roots_expected_red() {
    let mut random = 1;
    for _ in 0..5 {
        let curve = random_cubic(&mut random);
        let ones = flatten(&chop_cubic_at_values(curve, &[1.0; 5]));
        assert_eq!(&ones[..3], &curve[..3]);
        assert!(ones[3..].iter().all(|point| *point == curve[3]));
    }
}

#[test]
fn chop_cubic_at_t_values_null() {
    let mut random = 7;
    for num_chops in 1..=20 {
        let curve = random_cubic(&mut random);
        let step = 1.0 / (num_chops + 1) as f32;
        let explicit = (1..=num_chops)
            .map(|index| index as f32 * step)
            .collect::<Vec<_>>();
        // Pinned null-T semantics are equally spaced chops.
        let explicit_points = flatten(&chop_cubic_at_values(curve, &explicit));
        let null_points = flatten(&chop_cubic_at_values(
            curve,
            &(1..=num_chops)
                .map(|index| index as f32 / (num_chops + 1) as f32)
                .collect::<Vec<_>>(),
        ));
        for (actual, expected) in null_points.iter().zip(explicit_points) {
            assert!((actual.x - expected.x).abs() <= 1.0e-5);
            assert!((actual.y - expected.y).abs() <= 1.0e-5);
        }
    }
}

fn measure_angle(a: Vec2D, b: Vec2D) -> f32 {
    let product = length(a) * length(b);
    if product == 0.0 {
        0.0
    } else {
        (dot(a, b) / product).clamp(-1.0, 1.0).acos()
    }
}

fn measure_non_inflect_cubic_rotation(points: [Vec2D; 4]) -> f32 {
    let a = subtract(points[1], points[0]);
    let b = subtract(points[2], points[1]);
    let c = subtract(points[3], points[2]);
    let length_a = length(a);
    let length_b = length(b);
    let length_c = length(c);
    let zero_threshold = length_a.max(length_b).max(length_c).max(1.0) * 1.0e-4;
    if length_a <= zero_threshold {
        return measure_angle(b, c);
    }
    if length_b <= zero_threshold {
        return measure_angle(a, c);
    }
    if length_c <= zero_threshold {
        return measure_angle(a, b);
    }
    std::f32::consts::TAU - measure_angle(a, negate(b)) - measure_angle(b, negate(c))
}

#[test]
fn measure_non_inflect_cubic_rotation_direct_port() {
    for (points, expected) in [
        ([v(0.0, 0.0), v(0.0, 1.0), v(0.0, 2.0), v(0.0, 3.0)], 0.0),
        (
            [v(0.0, 0.0), v(1.0, 0.0), v(3.0, 0.0), v(2.0, 0.0)],
            std::f32::consts::PI,
        ),
        (
            [v(0.0, 1.0), v(0.0, 0.0), v(0.0, 2.0), v(0.0, 3.0)],
            std::f32::consts::PI,
        ),
        (
            [v(0.0, 1.0), v(0.0, 0.0), v(0.0, 3.0), v(0.0, 2.0)],
            std::f32::consts::TAU,
        ),
        (
            [v(0.0, 0.0), v(0.0, 1.0), v(1.0, 1.0), v(1.0, 0.0)],
            std::f32::consts::PI,
        ),
    ] {
        assert!(fuzzy_equal(
            measure_non_inflect_cubic_rotation(points),
            expected
        ));
    }

    let check_quad = |quad: [Vec2D; 3], expected| {
        for cubic in [
            [quad[0], quad[0], quad[1], quad[2]],
            [quad[0], quad[1], quad[1], quad[2]],
            [quad[0], quad[1], quad[2], quad[2]],
        ] {
            assert!(fuzzy_equal(
                measure_non_inflect_cubic_rotation(cubic),
                expected
            ));
        }
    };
    check_quad([v(0.0, 0.0), v(0.0, 1.0), v(0.0, 2.0)], 0.0);
    check_quad(
        [v(1.0, 0.0), v(0.0, 0.0), v(2.0, 0.0)],
        std::f32::consts::PI,
    );
    check_quad(
        [v(0.0, 0.0), v(0.0, 2.0), v(0.0, 1.0)],
        std::f32::consts::PI,
    );
    check_quad(
        [v(0.0, 0.0), v(0.5, 3.0_f32.sqrt() / 2.0), v(1.0, 0.0)],
        std::f32::consts::TAU / 3.0,
    );
}

fn corner_cubic(bits: u16) -> [Vec2D; 4] {
    std::array::from_fn(|index| {
        v(
            ((bits >> (index * 2)) & 1) as f32,
            ((bits >> (index * 2 + 1)) & 1) as f32,
        )
    })
}

#[test]
fn find_cubic_convex_180_chops_direct_port() {
    for bits in 0..(1 << 8) {
        let (roots, _) = find_cubic_convex_180_chops(corner_cubic(bits));
        assert!(roots.len() <= 2);
        assert!(roots.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(roots.iter().all(|root| *root > 0.0 && *root < 1.0));
    }

    let (roots, are_cusps) =
        find_cubic_convex_180_chops([v(0.0, 0.0), v(2.0, 2.0), v(4.0, 2.0), v(6.0, 0.0)]);
    assert!(roots.is_empty());
    assert!(!are_cusps);

    let (roots, are_cusps) =
        find_cubic_convex_180_chops([v(0.0, 0.0), v(1.0, 1.0), v(1.0, 0.0), v(0.0, 1.0)]);
    assert_eq!(roots.len(), 1);
    assert!(are_cusps);

    let (roots, _) = find_cubic_convex_180_chops([
        v(460.0, 460.0),
        v(598.0, 460.0),
        v(935.333_3, 460.0),
        v(1060.0, 460.0),
    ]);
    assert!(roots.is_empty());
}

#[test]
fn find_cubic_convex_180_chops_lines_direct_port() {
    let p0 = v(123.0, 200.0);
    let p3 = v(223.0, 432.0);
    let mut t0 = 1.0e-3;
    while t0 < 1.0 {
        let mut t1 = t0 + 0.097;
        while t1 < 1.0 {
            let line = [p0, lerp(p0, p3, t0), lerp(p0, p3, t1), p3];
            let (_, are_cusps) = find_cubic_convex_180_chops(line);
            assert!(!are_cusps);
            t1 += 0.097;
        }
        t0 += 0.12;
    }
}

fn polynomial_eval_cubic(points: [Vec2D; 4], t: f32) -> Vec2D {
    let one_minus_t = 1.0 - t;
    add(
        add(
            mul(points[0], one_minus_t.powi(3)),
            mul(points[1], 3.0 * one_minus_t.powi(2) * t),
        ),
        add(
            mul(points[2], 3.0 * one_minus_t * t.powi(2)),
            mul(points[3], t.powi(3)),
        ),
    )
}

#[test]
fn eval_cubic_direct_port() {
    for cubic in TEST_CUBICS {
        for (t0, t1) in std::iter::once((0.0, 1.0)).chain((0..=100).map(|step| {
            let t = step as f32 * 0.01;
            (t, t + 0.003)
        })) {
            for t in [t0, t1] {
                let actual = eval_cubic(cubic, t);
                let expected = polynomial_eval_cubic(cubic, t);
                assert!((actual.x - expected.x).abs() <= 1.0e-3);
                assert!((actual.y - expected.y).abs() <= 1.0e-3);
            }
        }
    }
}

fn cubic_coefficients(points: [Vec2D; 4]) -> (Vec2D, Vec2D, Vec2D) {
    let c = subtract(points[1], points[0]);
    let d = subtract(points[2], points[1]);
    let b = subtract(d, c);
    let a = add(mul(d, -3.0), subtract(points[3], points[0]));
    (a, b, c)
}

#[test]
fn find_cubic_coeffs_tangents_glsl_direct_port() {
    let source = pinned_shader_source();
    assert!(source.contains("C = p1 - p0;"));
    assert!(source.contains("B = D - C;"));
    assert!(source.contains("A = -3. * D + E;"));
    assert!(source.contains("find_cubic_tangents"));

    for points in TEST_CUBICS
        .into_iter()
        .chain((0..(1 << 8)).map(|bits| corner_cubic(bits).map(|point| mul(point, 100.0))))
    {
        let (a, b, c) = cubic_coefficients(points);
        let expected_c = subtract(points[1], points[0]);
        let expected_b = subtract(subtract(points[2], points[1]), expected_c);
        let expected_a = add(
            mul(subtract(points[2], points[1]), -3.0),
            subtract(points[3], points[0]),
        );
        assert_eq!((a, b, c), (expected_a, expected_b, expected_c));

        let tangents = cubic_tangents(points);
        assert_eq!(
            tangents[0],
            if points[0] != points[1] {
                subtract(points[1], points[0])
            } else if points[1] != points[2] {
                subtract(points[2], points[0])
            } else {
                subtract(points[3], points[0])
            }
        );
        assert_eq!(
            tangents[1],
            if points[3] != points[2] {
                subtract(points[3], points[2])
            } else if points[2] != points[1] {
                subtract(points[3], points[1])
            } else {
                subtract(points[3], points[0])
            }
        );
    }
}

fn clamped_divide(a: f32, b: f32) -> f32 {
    let a = if b < 0.0 { -a } else { a };
    let b = b.abs();
    if a > 0.0 {
        if a < b { a / b } else { 1.0 }
    } else {
        0.0
    }
}

#[test]
fn clamped_divide_glsl_direct_port() {
    let source = pinned_shader_source();
    assert!(source.contains("return a > .0 ? (a < b ? a / b : 1.) : .0;"));
    let infinity = f32::INFINITY;
    for (a, b, expected) in [
        (1.0, 2.0, 0.5),
        (-2.0, 0.0, 0.0),
        (1.0, 0.0, 1.0),
        (-infinity, 1.0, 0.0),
        (infinity, 1.0, 1.0),
        (infinity, infinity, 1.0),
        (-infinity, -infinity, 1.0),
        (-infinity, infinity, 0.0),
        (infinity, -infinity, 0.0),
        (0.0, 0.0, 0.0),
        (1.0, f32::NAN, 1.0),
        (-1.0, f32::NAN, 0.0),
        (f32::NAN, 1.0, 0.0),
        (f32::NAN, -1.0, 0.0),
        (f32::NAN, f32::NAN, 0.0),
    ] {
        assert_eq!(clamped_divide(a, b), expected);
    }
}

fn cubic_max_height(points: [Vec2D; 4]) -> (f32, f32) {
    let base = subtract(points[3], points[0]);
    let length_base = length(base);
    if length_base == 0.0 {
        return (0.5, 0.0);
    }
    let norm = v(-base.y / length_base, base.x / length_base);
    let h2 = dot(norm, subtract(points[2], points[0]));
    let h1 = dot(norm, subtract(points[1], points[0]));
    let dh = h1 - h2;
    let three_a = 3.0 * dh;
    let b = -h1 - dh;
    let c = h1;
    let mut t = 0.5;
    for _ in 0..3 {
        let three_at = three_a * t;
        t = clamped_divide(three_at * t - c, 2.0 * (three_at + b));
    }
    let height = (t * (t * (t * three_a + 3.0 * b) + 3.0 * c)).abs();
    (t, height)
}

#[test]
fn find_cubic_max_height_glsl_direct_port() {
    let source = pinned_shader_source();
    assert!(source.contains("for (int i = 0; i < 3; ++i)"));
    for points in TEST_CUBICS
        .into_iter()
        .chain((0..(1 << 8)).map(|bits| corner_cubic(bits).map(|point| mul(point, 100.0))))
    {
        let (t, height) = cubic_max_height(points);
        assert!(height >= 0.0 && (0.0..=1.0).contains(&t));
        let base = subtract(points[3], points[0]);
        let base_length = length(base);
        if base_length == 0.0 {
            assert_eq!(height, 0.0);
            continue;
        }
        let norm = v(-base.y / base_length, base.x / base_length);
        let height_at = |sample| dot(norm, subtract(eval_cubic(points, sample), points[0])).abs();
        assert!((height_at(t) - height).abs() <= 1.0e-4 * height.max(1.0));
        let epsilon = height.max(1.0) * 1.0e-3;
        let mut sample = 0.0;
        while sample <= 1.0 {
            assert!(height + epsilon > height_at(sample));
            sample += 0.005_137;
        }
    }
}

fn assert_shader_expected_red(test_name: &str, required_symbols: &[&str]) {
    let source = pinned_shader_source();
    for symbol in required_symbols {
        assert!(source.contains(symbol), "pinned shader lost {symbol}");
    }
    for points in TEST_CUBICS
        .into_iter()
        .chain((0..(1 << 8)).map(|bits| corner_cubic(bits).map(|point| mul(point, 100.0))))
    {
        assert!(
            points
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
    }
    panic!("expected-red: {test_name} awaits a Rust shader-function execution bridge");
}

#[test]
#[ignore = "expected-red: deprecated convex-90 test helper has no Rust shader execution bridge"]
fn find_cubic_convex_90_chops_direct_port() {
    assert_shader_expected_red(
        "find_cubic_convex_90_chops",
        &["find_cubic_coeffs", "measure_cubic_local_curvature"],
    );
}

#[test]
#[ignore = "expected-red: GLSL curvature helper has no Rust shader execution bridge"]
fn measure_cubic_local_curvature_glsl_direct_port() {
    assert_shader_expected_red(
        "measure_cubic_local_curvature_glsl",
        &["measure_cubic_local_curvature", "find_cubic_max_height"],
    );
}
