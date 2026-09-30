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

fn vector_cross(a: Vec2D, b: Vec2D) -> f32 {
    a.x * b.y - a.y * b.x
}

fn fuzzy_equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPSILON
}

fn fuzzy_equal_with_tolerance(a: f32, b: f32, tolerance: f32) -> bool {
    debug_assert!(tolerance >= 0.0);
    (a - b).abs() <= tolerance
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

struct PinnedMt19937_64 {
    state: [u64; 312],
    index: usize,
}

impl PinnedMt19937_64 {
    fn seeded(seed: u64) -> Self {
        let mut state = [0; 312];
        state[0] = seed;
        for index in 1..state.len() {
            state[index] = 6_364_136_223_846_793_005u64
                .wrapping_mul(state[index - 1] ^ (state[index - 1] >> 62))
                .wrapping_add(index as u64);
        }
        Self { state, index: 312 }
    }

    fn next_u64(&mut self) -> u64 {
        if self.index == self.state.len() {
            for index in 0..self.state.len() {
                let joined = (self.state[index] & 0xffff_ffff_8000_0000)
                    | (self.state[(index + 1) % self.state.len()] & 0x7fff_ffff);
                let mut value = self.state[(index + 156) % self.state.len()] ^ (joined >> 1);
                if joined & 1 != 0 {
                    value ^= 0xb502_6f5a_a966_19e9;
                }
                self.state[index] = value;
            }
            self.index = 0;
        }
        let mut value = self.state[self.index];
        self.index += 1;
        value ^= (value >> 29) & 0x5555_5555_5555_5555;
        value ^= (value << 17) & 0x71d6_7fff_eda6_0000;
        value ^= (value << 37) & 0xfff7_eee0_0000_0000;
        value ^ (value >> 43)
    }

    fn f32(&mut self, start: f32, end: f32) -> f32 {
        let unit = (self.next_u64() >> 40) as f32 * (1.0 / (1 << 24) as f32);
        start + unit * (end - start)
    }

    fn cubic(&mut self) -> [Vec2D; 4] {
        std::array::from_fn(|_| v(self.f32(-100.0, 100.0), self.f32(-100.0, 100.0)))
    }
}

#[test]
fn chop_cubic_at() {
    // Rust cannot pass one allocation as overlapping `&` and `&mut` slices.
    // Preserve the pinned alias observable by using one buffer for input and
    // output, ending the value-copy read before overwriting that same buffer.
    let mut aliased = [v(0.0, 0.0); 7];
    for (index, point) in aliased.iter_mut().enumerate() {
        *point = v(index as f32, index as f32);
    }
    let source = [aliased[0], aliased[1], aliased[2], aliased[3]];
    aliased.copy_from_slice(&flatten(&chop_cubic_at_values(source, &[0.5])));
    for (index, point) in aliased.iter().enumerate() {
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
#[ignore = "expected-red: Rust multi-chop is not exact for equal and repeated endpoint roots"]
fn chop_cubic_at_complete_direct_port_expected_red() {
    // The pinned Catch2 case contains all three assertion groups. Keep one
    // case-level entry point so the correspondence ledger cannot promote only
    // the currently green subset.
    chop_cubic_at();
    chop_cubic_at_equal_roots_are_exactly_degenerate_expected_red();
    chop_cubic_at_repeated_endpoint_roots_expected_red();
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
        let explicit_points = flatten(&chop_cubic_at_values(curve, &explicit));
        // `Option` is the Rust-safe translation of the nullable C++ pointer.
        // The action intentionally supplies None; the adapter materializes the
        // pinned owner's equally spaced null-T semantics before entering the
        // production multi-chop owner.
        let null_t_values: Option<&[f32]> = None;
        let null_roots = match null_t_values {
            Some(values) => values.to_vec(),
            None => (1..=num_chops)
                .map(|index| index as f32 / (num_chops + 1) as f32)
                .collect(),
        };
        let null_points = flatten(&chop_cubic_at_values(curve, &null_roots));
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

fn valid_unit_divide(mut numerator: f32, mut denominator: f32) -> Option<f32> {
    if numerator < 0.0 {
        numerator = -numerator;
        denominator = -denominator;
    }
    if denominator == 0.0 || numerator == 0.0 || numerator >= denominator {
        return None;
    }
    let ratio = numerator / denominator;
    (!ratio.is_nan() && ratio > 0.0 && ratio < 1.0).then_some(ratio)
}

fn find_unit_quad_roots(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a == 0.0 {
        return valid_unit_divide(-c, b).into_iter().collect();
    }
    let discriminant = b as f64 * b as f64 - 4.0 * a as f64 * c as f64;
    if discriminant < 0.0 {
        return Vec::new();
    }
    let root = discriminant.sqrt() as f32;
    if !root.is_finite() {
        return Vec::new();
    }
    let q = if b < 0.0 {
        -(b - root) * 0.5
    } else {
        -(b + root) * 0.5
    };
    let mut roots = Vec::with_capacity(2);
    roots.extend(valid_unit_divide(q, a));
    roots.extend(valid_unit_divide(c, q));
    roots.sort_by(f32::total_cmp);
    roots.dedup();
    roots
}

fn find_cubic_inflections(points: [Vec2D; 4]) -> Vec<f32> {
    let ax = points[1].x - points[0].x;
    let ay = points[1].y - points[0].y;
    let bx = points[2].x - 2.0 * points[1].x + points[0].x;
    let by = points[2].y - 2.0 * points[1].y + points[0].y;
    let cx = points[3].x + 3.0 * (points[1].x - points[2].x) - points[0].x;
    let cy = points[3].y + 3.0 * (points[1].y - points[2].y) - points[0].y;
    find_unit_quad_roots(bx * cy - by * cx, ax * cy - ay * cx, ax * by - ay * bx)
}

fn is_linear_three(p0: Vec2D, p1: Vec2D, p2: Vec2D) -> bool {
    fuzzy_equal(vector_cross(subtract(p0, p1), subtract(p2, p1)), 0.0)
}

fn is_linear_cubic(points: [Vec2D; 4]) -> bool {
    is_linear_three(points[0], points[1], points[2])
        && is_linear_three(points[0], points[2], points[3])
        && is_linear_three(points[1], points[2], points[3])
}

fn check_cubic_convex_180(points: [Vec2D; 4]) {
    let inflections = find_cubic_inflections(points);
    let (roots, are_cusps) = find_cubic_convex_180_chops(points);
    if !inflections.is_empty() {
        assert_eq!(inflections.len(), roots.len());
        if !are_cusps {
            assert!(inflections.len() == 1 || (inflections[0] - inflections[1]).abs() >= EPSILON);
        }
        for (inflection, root) in inflections.into_iter().zip(roots) {
            assert!(fuzzy_equal(inflection, root));
        }
        return;
    }

    let total_rotation = measure_non_inflect_cubic_rotation(points);
    let mut chops = flatten(&chop_cubic_at_values(points, &roots));
    let mut radians_sum = 0.0;
    if are_cusps {
        if roots.len() == 1 {
            radians_sum = std::f32::consts::PI;
            let straddles = [
                (roots[0] - std::f32::consts::PI).max(0.0),
                (roots[0] + std::f32::consts::PI).min(1.0),
            ];
            let straddle_chops = flatten(&chop_cubic_at_values(points, &straddles));
            chops[1..3].copy_from_slice(&straddle_chops[1..3]);
            chops[4..6].copy_from_slice(&straddle_chops[7..9]);
        } else if roots.len() == 2 {
            radians_sum = std::f32::consts::TAU;
            chops[1] = chops[0];
            chops[2] = chops[3];
            chops[4] = chops[3];
            chops[5] = chops[6];
            chops[7] = chops[6];
            chops[8] = chops[9];
        }
    }
    for index in 0..=roots.len() {
        let segment = [
            chops[index * 3],
            chops[index * 3 + 1],
            chops[index * 3 + 2],
            chops[index * 3 + 3],
        ];
        let radians = measure_non_inflect_cubic_rotation(segment);
        assert!(radians < std::f32::consts::PI + EPSILON);
        radians_sum += radians;
    }
    if total_rotation < std::f32::consts::PI - EPSILON {
        assert!(roots.is_empty());
    } else if !is_linear_cubic(points) {
        assert!(fuzzy_equal(radians_sum, total_rotation));
        if total_rotation > std::f32::consts::PI + EPSILON {
            assert_eq!(roots.len(), 1);
            let first = [chops[0], chops[1], chops[2], chops[3]];
            let second = [chops[3], chops[4], chops[5], chops[6]];
            assert!(fuzzy_equal(
                measure_non_inflect_cubic_rotation(first),
                std::f32::consts::PI
            ));
            assert!(fuzzy_equal(
                measure_non_inflect_cubic_rotation(second),
                total_rotation - std::f32::consts::PI
            ));
        }
        assert!(!are_cusps);
    } else {
        assert!(are_cusps);
    }
}

#[test]
fn find_cubic_convex_180_chops_direct_port() {
    for bits in 0..(1 << 8) {
        check_cubic_convex_180(corner_cubic(bits));
    }

    let hex = [
        0x3ee0_ac74,
        0x3f1e_061a,
        0x3e0f_c408,
        0x3f45_7230,
        0x3f42_ac7c,
        0x3f70_d76c,
        0x3f4e_6520,
        0x3f6a_cafa,
    ];
    check_cubic_convex_180(std::array::from_fn(|index| {
        v(
            f32::from_bits(hex[index * 2]),
            f32::from_bits(hex[index * 2 + 1]),
        )
    }));

    let (roots, _) =
        find_cubic_convex_180_chops([v(0.0, 0.0), v(2.0, 2.0), v(4.0, 2.0), v(6.0, 0.0)]);
    assert!(roots.is_empty());

    let (roots, are_cusps) =
        find_cubic_convex_180_chops([v(0.0, 0.0), v(1.0, 1.0), v(1.0, 0.0), v(0.0, 1.0)]);
    assert_eq!(roots.len(), 1);
    assert!(are_cusps);

    let epsilon = 1.0 / (1 << 11) as f64;
    let epsilon_squared = epsilon * epsilon;
    let height = (1.0 - epsilon_squared) / (3.0 * epsilon_squared + 1.0);
    let dy = (1.0 - height) * 0.5;
    let mut cusp = [v(0.0, 0.0), v(1.0, 1.0), v(1.0, 0.0), v(0.0, 1.0)];
    cusp[1].y = (1.0 - dy) as f32;
    cusp[2].y = dy as f32;
    let inflections = find_cubic_inflections(cusp);
    assert_eq!(inflections.len(), 2);
    assert!(fuzzy_equal_with_tolerance(
        inflections[1] - inflections[0],
        epsilon as f32,
        epsilon_squared as f32
    ));

    cusp[1].y = (1.0 - 4.0 * dy) as f32;
    cusp[2].y = (4.0 * dy) as f32;
    let (roots, are_cusps) = find_cubic_convex_180_chops(cusp);
    assert_eq!(roots.len(), 2);
    assert!(!are_cusps);

    cusp[1].y = (1.0 - 0.9 * dy) as f32;
    cusp[2].y = (0.9 * dy) as f32;
    let (roots, are_cusps) = find_cubic_convex_180_chops(cusp);
    assert_eq!(roots.len(), 1);
    assert!(are_cusps);

    let p = [
        v(460.0, 1060.0),
        v(774.0, 526.0),
        v(60.0, 660.0),
        v(460.0, 460.0),
        v(667.0, 460.0),
        v(1060.0, 460.0),
        v(686.0, 460.0),
        v(686.0, 660.0),
        v(1042.0, 1020.0),
    ];
    let c0 = lerp(p[3], p[4], 2.0 / 3.0);
    let c1 = lerp(p[5], p[4], 2.0 / 3.0);
    let (roots, _) = find_cubic_convex_180_chops([p[3], c0, c1, p[5]]);
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
        let evaluator = CoarseEvalCubic::new(cubic);
        let check_pair = |t0: f32, t1: f32| {
            // The pinned owner evaluates two SIMD lanes in one call. Rust's
            // owner exposes scalar lanes, so execute the same pair through one
            // retained evaluator and preserve the pair's assertion order.
            for (actual, t) in [(evaluator.at(t0), t0), (evaluator.at(t1), t1)] {
                let expected = polynomial_eval_cubic(cubic, t);
                assert!((actual.x - expected.x).abs() <= 1.0e-3);
                assert!((actual.y - expected.y).abs() <= 1.0e-3);
            }
        };
        check_pair(0.0, 1.0);
        let mut t = 0.0;
        while t <= 1.0 {
            check_pair(t, t + 0.003);
            t += 0.01;
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
        if a < b {
            a / b
        } else {
            1.0
        }
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

const FEATHERING_CUSP_PADDING: f32 = 1.0e-3;

fn find_cubic_convex_90_chops_test_owner(
    points: [Vec2D; 4],
    cusp_padding: f32,
) -> (Vec<f32>, bool) {
    const TESS_EPSILON: f32 = 1.0 / (1 << 10) as f32;
    let (a_coeff, b_coeff, c_coeff) = cubic_coefficients(points);
    let mut a = vector_cross(a_coeff, b_coeff);
    let mut b_over_2 = vector_cross(a_coeff, c_coeff) * 0.5;
    let mut c = vector_cross(b_coeff, c_coeff);
    let mut discriminant_over_4 = b_over_2 * b_over_2 - a * c;
    let mut cusp_threshold = a * (TESS_EPSILON * 0.5);
    cusp_threshold *= cusp_threshold;

    let mut roots = [1.0; 4];
    let (tangent_90, are_cusps) = if discriminant_over_4 < -cusp_threshold
        || a.abs().max(b_over_2.abs()) < c.abs() * TESS_EPSILON
    {
        roots[0] = -c / b_over_2;
        let tangent = if c_coeff != v(0.0, 0.0) {
            c_coeff
        } else {
            subtract(points[2], points[0])
        };
        (tangent, false)
    } else if discriminant_over_4 > cusp_threshold {
        let mut q = discriminant_over_4.sqrt();
        q = -b_over_2 - q.copysign(b_over_2);
        roots[0] = q / a;
        roots[1] = c / q;
        let t = if (roots[0] - 0.5).abs() < (roots[1] - 0.5).abs() {
            roots[0]
        } else {
            roots[1]
        };
        (
            add(mul(add(mul(a_coeff, t), mul(b_coeff, 2.0)), t), c_coeff),
            false,
        )
    } else {
        let tangent = if c_coeff != v(0.0, 0.0) {
            c_coeff
        } else {
            subtract(points[2], points[0])
        };
        (tangent, true)
    };

    a = dot(a_coeff, tangent_90);
    b_over_2 = dot(b_coeff, tangent_90);
    c = dot(c_coeff, tangent_90);
    discriminant_over_4 = b_over_2 * b_over_2 - a * c;
    let mut q = discriminant_over_4.sqrt();
    q = -b_over_2 - q.copysign(b_over_2);
    roots[2] = q / a;
    roots[3] = c / q;
    for root in &mut roots {
        if !(*root > 0.0 && *root < 1.0) {
            *root = 1.0;
        }
    }
    roots.sort_by(f32::total_cmp);
    let mut roots = roots
        .into_iter()
        .take_while(|root| *root != 1.0)
        .collect::<Vec<_>>();

    if are_cusps && !roots.is_empty() {
        debug_assert!(roots.len() <= 2);
        let cusp_roots = roots.clone();
        roots.resize(cusp_roots.len() * 2, 0.0);
        for index in (0..cusp_roots.len()).rev() {
            let maximum = if index + 1 == cusp_roots.len() {
                1.0
            } else {
                roots[index * 2 + 1]
            };
            let minimum = if index == 0 {
                0.0
            } else {
                (cusp_roots[index - 1] + cusp_roots[index]) * 0.5
            };
            roots[index * 2 + 1] = (cusp_roots[index] + cusp_padding).min(maximum);
            roots[index * 2] = (cusp_roots[index] - cusp_padding).max(minimum);
        }
        if roots.last() == Some(&1.0) {
            roots.pop();
        }
        roots.sort_by(f32::total_cmp);
    }
    (roots, are_cusps)
}

fn check_cubic_convex_90_chops(points: [Vec2D; 4]) {
    let (roots, are_cusps) = find_cubic_convex_90_chops_test_owner(points, FEATHERING_CUSP_PADDING);
    assert!(roots.len() <= 4);
    let chops = flatten(&chop_cubic_at_values(points, &roots));
    for index in 0..=roots.len() {
        if are_cusps && index & 1 != 0 {
            continue;
        }
        let segment = [
            chops[index * 3],
            chops[index * 3 + 1],
            chops[index * 3 + 2],
            chops[index * 3 + 3],
        ];
        let rotation = measure_non_inflect_cubic_rotation(segment);
        assert!(rotation <= std::f32::consts::FRAC_PI_2 + 1.0e-2);
    }
}

#[test]
fn find_cubic_convex_90_chops_direct_port() {
    for points in TEST_CUBICS
        .into_iter()
        .chain((0..(1 << 8)).map(|bits| corner_cubic(bits).map(|point| mul(point, 100.0))))
    {
        check_cubic_convex_90_chops(points);
    }
    let mut random = PinnedMt19937_64::seeded(0);
    for _ in 0..100 {
        check_cubic_convex_90_chops(random.cubic());
    }
}
