use super::{math_types, vec2d::Vec2D};

#[derive(Clone, Copy, Debug)]
pub struct CubicCoeffs {
    pub a: Vec2D,
    pub b: Vec2D,
    pub c: Vec2D,
}
impl CubicCoeffs {
    pub fn new(points: &[Vec2D; 4]) -> Self {
        let c = points[1] - points[0];
        let d = points[2] - points[1];
        let e = points[3] - points[0];
        Self {
            a: Vec2D::scale_and_add(e, d, -3.0),
            b: d - c,
            c,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EvalCubic {
    a: Vec2D,
    b: Vec2D,
    c: Vec2D,
    d: Vec2D,
}
impl EvalCubic {
    pub fn new(points: &[Vec2D; 4]) -> Self {
        Self::from_coeffs(CubicCoeffs::new(points), points[0])
    }
    pub fn from_coeffs(coeffs: CubicCoeffs, p0: Vec2D) -> Self {
        Self {
            a: coeffs.a,
            b: 3.0 * coeffs.b,
            c: 3.0 * coeffs.c,
            d: p0,
        }
    }
    pub fn at(self, t: f32) -> Vec2D {
        let value = Vec2D::scale_and_add(self.b, self.a, t);
        let value = Vec2D::scale_and_add(self.c, value, t);
        Vec2D::scale_and_add(self.d, value, t)
    }
    pub fn at_pair(self, t: [f32; 2]) -> [Vec2D; 2] {
        [self.at(t[0]), self.at(t[1])]
    }
}

pub fn calc_polar_segments_per_radian<const PRECISION: i32>(approx_dev_stroke_radius: f32) -> f32 {
    let cos_theta = 1.0 - (1.0 / PRECISION as f32) / approx_dev_stroke_radius;
    0.5 / cpp_max(cos_theta, -1.0).acos()
}
pub fn eval_cubic_at(points: &[Vec2D; 4], t: f32) -> Vec2D {
    EvalCubic::new(points).at(t)
}

fn mix(a: Vec2D, b: Vec2D, t: f32) -> Vec2D {
    Vec2D::lerp(a, b, t)
}

pub fn chop_cubic_at(src: &[Vec2D; 4], dst: &mut [Vec2D; 7], t: f32) {
    assert!((0.0..=1.0).contains(&t));
    if t == 1.0 {
        dst[..4].copy_from_slice(src);
        dst[4] = src[3];
        dst[5] = src[3];
        dst[6] = src[3];
        return;
    }
    let ab = mix(src[0], src[1], t);
    let bc = mix(src[1], src[2], t);
    let cd = mix(src[2], src[3], t);
    let abc = mix(ab, bc, t);
    let bcd = mix(bc, cd, t);
    let abcd = mix(abc, bcd, t);
    *dst = [src[0], ab, abc, abcd, bcd, cd, src[3]];
}
pub fn chop_cubic_at_two(src: &[Vec2D; 4], dst: &mut [Vec2D; 10], t0: f32, t1: f32) {
    assert!(0.0 <= t0 && t0 <= t1 && t1 <= 1.0);
    if t1 == 1.0 {
        let mut first = [Vec2D::default(); 7];
        chop_cubic_at(src, &mut first, t0);
        dst[..7].copy_from_slice(&first);
        dst[7] = src[3];
        dst[8] = src[3];
        dst[9] = src[3];
        return;
    }
    let ab0 = mix(src[0], src[1], t0);
    let bc0 = mix(src[1], src[2], t0);
    let cd0 = mix(src[2], src[3], t0);
    let abc0 = mix(ab0, bc0, t0);
    let bcd0 = mix(bc0, cd0, t0);
    let p0 = mix(abc0, bcd0, t0);
    let ab1 = mix(src[0], src[1], t1);
    let bc1 = mix(src[1], src[2], t1);
    let cd1 = mix(src[2], src[3], t1);
    let abc1 = mix(ab1, bc1, t1);
    let bcd1 = mix(bc1, cd1, t1);
    let p1 = mix(abc1, bcd1, t1);
    let middle0 = mix(abc0, bcd0, t1);
    let middle1 = mix(abc1, bcd1, t0);
    *dst = [
        src[0], ab0, abc0, p0, middle0, middle1, p1, bcd1, cd1, src[3],
    ];
}
pub fn chop_cubic_at_values(
    src: &[Vec2D; 4],
    dst: &mut [Vec2D],
    t_values: Option<&[f32]>,
    t_count: usize,
) {
    if let Some(values) = t_values {
        assert!(values.len() >= t_count);
        assert!(values[..t_count].iter().all(|t| *t >= 0.0 && *t <= 1.0));
        assert!(values[..t_count].windows(2).all(|w| w[0] <= w[1]));
    }
    if t_count == 0 {
        dst[..4].copy_from_slice(src);
        return;
    }
    assert!(dst.len() >= 3 * (t_count + 1) + 1);
    let mut current = *src;
    let mut offset = 0;
    let mut index = 0;
    let mut last_t = 0.0;
    while index + 1 < t_count {
        let pair = if let Some(values) = t_values {
            let p = [
                math_types::clamp((values[index] - last_t) / (1.0 - last_t), 0.0, 1.0),
                math_types::clamp((values[index + 1] - last_t) / (1.0 - last_t), 0.0, 1.0),
            ];
            last_t = values[index + 1];
            p
        } else {
            [
                1.0 / (t_count + 1 - index) as f32,
                2.0 / (t_count + 1 - index) as f32,
            ]
        };
        let mut chopped = [Vec2D::default(); 10];
        chop_cubic_at_two(&current, &mut chopped, pair[0], pair[1]);
        dst[offset..offset + 10].copy_from_slice(&chopped);
        current.copy_from_slice(&chopped[6..10]);
        offset += 6;
        index += 2;
    }
    if index < t_count {
        let mut t = t_values.map_or(0.5, |values| values[index]);
        t = math_types::clamp((t - last_t) / (1.0 - last_t), 0.0, 1.0);
        let mut chopped = [Vec2D::default(); 7];
        chop_cubic_at(&current, &mut chopped, t);
        dst[offset..offset + 7].copy_from_slice(&chopped);
    }
}

pub fn measure_angle_between_vectors(a: Vec2D, b: Vec2D) -> f32 {
    let mut cosine = Vec2D::dot(a, b) / (Vec2D::dot(a, a) * Vec2D::dot(b, b)).sqrt();
    cosine = cpp_max(cpp_min(1.0, cosine), -1.0);
    cosine.acos()
}

const TESS_EPSILON: f32 = 1.0 / 1024.0;
const IEEE_ONE_MINUS_2_EPSILON: u32 = (127 << 23) - 2 * (1 << (24 - 10));
const _: () = assert!(f32::from_bits(IEEE_ONE_MINUS_2_EPSILON) == 1.0 - 2.0 * TESS_EPSILON);
pub fn find_cubic_convex_180_chops(
    points: &[Vec2D; 4],
    output: &mut [f32; 2],
    are_cusps: &mut bool,
) -> usize {
    let e01 = points[1] - points[0];
    let e12 = points[2] - points[1];
    let e23 = points[3] - points[2];
    // SIMD computes both product lanes before reducing; do not fuse them.
    let simd_cross = |a: Vec2D, b: Vec2D| a.x * b.y - a.y * b.x;
    let simd_length_squared = |edge: Vec2D| edge.x * edge.x + edge.y * edge.y;
    let mut c01x12 = simd_cross(e01, e12);
    let mut c12x23 = simd_cross(e12, e23);
    let mut c01x23 = simd_cross(e01, e23);
    let e01_is_zero = e01.x == 0.0 && e01.y == 0.0;
    let e23_is_zero = e23.x == 0.0 && e23.y == 0.0;

    // A zero first or last edge leaves only one relevant triangle turn.
    if e01_is_zero {
        c01x12 = c12x23;
        c01x23 = c12x23;
    }
    if e23_is_zero {
        c12x23 = c01x12;
        c01x23 = c01x12;
    }

    let turn_min = cpp_min(cpp_min(c01x12, c12x23), c01x23);
    let turn_max = cpp_max(cpp_max(c01x12, c12x23), c01x23);
    let max_edge_length_squared = cpp_max(
        cpp_max(simd_length_squared(e01), simd_length_squared(e12)),
        simd_length_squared(e23),
    );
    let turn_threshold = math_types::EPSILON * max_edge_length_squared;

    // Same winding for all edge pairs means convex and less than 180 degrees.
    let is_convex = turn_min > turn_threshold || turn_max < -turn_threshold;
    let is_line = e01_is_zero && e23_is_zero && (e12.x != 0.0 || e12.y != 0.0);
    if is_convex || is_line {
        *are_cusps = false;
        return 0;
    }

    let coeffs = CubicCoeffs::new(points);
    let mut a = Vec2D::cross(coeffs.a, coeffs.b);
    let mut b_over_minus_2 = -0.5 * Vec2D::cross(coeffs.a, coeffs.c);
    let mut c = Vec2D::cross(coeffs.b, coeffs.c);
    let mut discriminant_over_4 = b_over_minus_2 * b_over_minus_2 - a * c;
    let cusp_threshold = (a * (TESS_EPSILON / 2.0)).powi(2);
    if discriminant_over_4 < -cusp_threshold {
        *are_cusps = false;
        let root = c / b_over_minus_2;
        if (root - TESS_EPSILON).to_bits() < IEEE_ONE_MINUS_2_EPSILON {
            output[0] = root;
            return 1;
        }
        return 0;
    }
    *are_cusps = discriminant_over_4 <= cusp_threshold;
    if *are_cusps {
        if a != 0.0 || b_over_minus_2 != 0.0 || c != 0.0 {
            let root = b_over_minus_2 / a;
            if (root - TESS_EPSILON).to_bits() < IEEE_ONE_MINUS_2_EPSILON {
                output[0] = root;
                return 1;
            }
            *are_cusps = false;
            return 0;
        }
        let base = points[3] - points[0];
        let dots = points.map(|point| Vec2D::dot(point, base));
        if dots[1] > dots[0] && dots[2] > dots[1] && dots[3] > dots[2] {
            *are_cusps = false;
            return 0;
        }
        let tangent0 = if coeffs.c.x != 0.0 || coeffs.c.y != 0.0 {
            coeffs.c
        } else {
            points[2] - points[0]
        };
        a = Vec2D::dot(tangent0, coeffs.a);
        b_over_minus_2 = -Vec2D::dot(tangent0, coeffs.b);
        c = Vec2D::dot(tangent0, coeffs.c);
        discriminant_over_4 = cpp_max(b_over_minus_2 * b_over_minus_2 - a * c, 0.0);
    }
    let mut q = discriminant_over_4.sqrt().copysign(b_over_minus_2);
    q += b_over_minus_2;
    let mut roots = [q / a, c / q];
    let inside = roots.map(|root| root > TESS_EPSILON && root < 1.0 - TESS_EPSILON);
    if inside[0] {
        if inside[1] && roots[0] != roots[1] {
            if roots[0] > roots[1] {
                roots.swap(0, 1);
            }
            *output = roots;
            return 2;
        }
        output[0] = roots[0];
        return 1;
    }
    if inside[1] {
        output[0] = roots[1];
        return 1;
    }
    0
}
pub fn find_cubic_tan0(points: &[Vec2D; 4]) -> Vec2D {
    (if points[0] != points[1] {
        points[1]
    } else if points[1] != points[2] {
        points[2]
    } else {
        points[3]
    }) - points[0]
}
pub fn find_cubic_tan1(points: &[Vec2D; 4]) -> Vec2D {
    points[3]
        - if points[3] != points[2] {
            points[2]
        } else if points[2] != points[1] {
            points[1]
        } else {
            points[0]
        }
}
pub fn find_cubic_tangents(points: &[Vec2D; 4]) -> [Vec2D; 2] {
    [find_cubic_tan0(points), find_cubic_tan1(points)]
}
pub const fn pow2(value: f32) -> f32 {
    value * value
}
pub const fn pow3(value: f32) -> f32 {
    value * pow2(value)
}
pub const fn length_pow2(value: Vec2D) -> f32 {
    pow2(value.x) + pow2(value.y)
}
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

use super::simd::{self, Float4, GVec, Int4};

#[derive(Clone, Copy, Debug)]
pub struct Convex180Chops4 {
    pub num_chops: Int4,
    pub t0: Float4,
    pub t1: Float4,
    pub possible_cusps: Int4,
}
fn mask4(mut predicate: impl FnMut(usize) -> bool) -> Int4 {
    GVec::from_array(core::array::from_fn(|i| if predicate(i) { -1 } else { 0 }))
}
fn cross4(ax: Float4, ay: Float4, bx: Float4, by: Float4) -> Float4 {
    let axby = ax * by;
    let aybx = ay * bx;
    axby - aybx
}
fn length_squared4(x: Float4, y: Float4) -> Float4 {
    let xx = x * x;
    let yy = y * y;
    xx + yy
}
pub fn cubics_need_no_chops(
    p0x: Float4,
    p0y: Float4,
    p1x: Float4,
    p1y: Float4,
    p2x: Float4,
    p2y: Float4,
    p3x: Float4,
    p3y: Float4,
) -> Int4 {
    let e01x = p1x - p0x;
    let e01y = p1y - p0y;
    let e12x = p2x - p1x;
    let e12y = p2y - p1y;
    let e23x = p3x - p2x;
    let e23y = p3y - p2y;
    let mut c01x12 = cross4(e01x, e01y, e12x, e12y);
    let mut c12x23 = cross4(e12x, e12y, e23x, e23y);
    let mut c01x23 = cross4(e01x, e01y, e23x, e23y);
    let e01zero = mask4(|i| e01x[i] == 0.0 && e01y[i] == 0.0);
    let e23zero = mask4(|i| e23x[i] == 0.0 && e23y[i] == 0.0);
    c01x12 = simd::if_then_else(e01zero, c12x23, c01x12);
    c01x23 = simd::if_then_else(e01zero, c12x23, c01x23);
    c12x23 = simd::if_then_else(e23zero, c01x12, c12x23);
    c01x23 = simd::if_then_else(e23zero, c01x12, c01x23);
    let turn_min = simd::min_fast(simd::min_fast(c01x12, c12x23), c01x23);
    let turn_max = simd::max_fast(simd::max_fast(c01x12, c12x23), c01x23);
    let max_length = simd::max_fast(
        simd::max_fast(length_squared4(e01x, e01y), length_squared4(e12x, e12y)),
        length_squared4(e23x, e23y),
    );
    let threshold = GVec::splat(math_types::EPSILON) * max_length;
    let convex = mask4(|i| turn_min[i] > threshold[i] || turn_max[i] < -threshold[i]);
    let line = e01zero & e23zero & mask4(|i| e12x[i] != 0.0 || e12y[i] != 0.0);
    convex | line
}
pub fn find_cubic_convex_180_chops4(
    p0x: Float4,
    p0y: Float4,
    p1x: Float4,
    p1y: Float4,
    p2x: Float4,
    p2y: Float4,
    p3x: Float4,
    p3y: Float4,
) -> Convex180Chops4 {
    let cx = p1x - p0x;
    let cy = p1y - p0y;
    let dx = p2x - p1x;
    let dy = p2y - p1y;
    let ex = p3x - p0x;
    let ey = p3y - p0y;
    let bx = dx - cx;
    let by = dy - cy;
    let ax = GVec::splat(-3.0) * dx + ex;
    let ay = GVec::splat(-3.0) * dy + ey;
    let a = cross4(ax, ay, bx, by);
    let b = cross4(ax, ay, cx, cy);
    let c = cross4(bx, by, cx, cy);
    let bm = GVec::splat(-0.5) * b;
    let discr = bm * bm - a * c;
    let cusp = a * GVec::splat(TESS_EPSILON / 2.0);
    let cusp = cusp * cusp;
    let no_inflections = mask4(|i| discr[i] < -cusp[i]);
    let root180 = c / bm;
    let chop180 = mask4(|i| (root180[i] - TESS_EPSILON).to_bits() < IEEE_ONE_MINUS_2_EPSILON);
    if simd::all(no_inflections) {
        return Convex180Chops4 {
            num_chops: chop180 & GVec::splat(1),
            t0: root180,
            t1: GVec::splat(0.0),
            possible_cusps: GVec::splat(0),
        };
    }
    let possible_cusps = mask4(|i| no_inflections[i] == 0 && discr[i] <= cusp[i]);
    let q = simd::sqrt(discr);
    let q = GVec::from_array(core::array::from_fn(|i| q[i].copysign(bm[i]))) + bm;
    let r0 = q / a;
    let r1 = c / q;
    let in0 = mask4(|i| r0[i] > TESS_EPSILON && r0[i] < 1.0 - TESS_EPSILON);
    let in1 = mask4(|i| r1[i] > TESS_EPSILON && r1[i] < 1.0 - TESS_EPSILON);
    let two = in0 & in1 & mask4(|i| r0[i] != r1[i]);
    let swap = mask4(|i| r0[i] > r1[i]);
    let general_t0 = simd::if_then_else(
        two,
        simd::if_then_else(swap, r1, r0),
        simd::if_then_else(in0, r0, r1),
    );
    let general_t1 = simd::if_then_else(swap, r0, r1);
    let general_count = simd::if_then_else(two, GVec::splat(2), (in0 | in1) & GVec::splat(1));
    Convex180Chops4 {
        num_chops: simd::if_then_else(no_inflections, chop180 & GVec::splat(1), general_count),
        t0: simd::if_then_else(no_inflections, root180, general_t0),
        t1: general_t1,
        possible_cusps,
    }
}
pub fn find_cubic_tangents4(
    p0x: Float4,
    p0y: Float4,
    p1x: Float4,
    p1y: Float4,
    p2x: Float4,
    p2y: Float4,
    p3x: Float4,
    p3y: Float4,
) -> [Float4; 4] {
    let differ01 = mask4(|i| p0x[i] != p1x[i] || p0y[i] != p1y[i]);
    let differ12 = mask4(|i| p1x[i] != p2x[i] || p1y[i] != p2y[i]);
    let differ23 = mask4(|i| p2x[i] != p3x[i] || p2y[i] != p3y[i]);
    [
        simd::if_then_else(differ01, p1x, simd::if_then_else(differ12, p2x, p3x)) - p0x,
        simd::if_then_else(differ01, p1y, simd::if_then_else(differ12, p2y, p3y)) - p0y,
        p3x - simd::if_then_else(differ23, p2x, simd::if_then_else(differ12, p1x, p0x)),
        p3y - simd::if_then_else(differ23, p2y, simd::if_then_else(differ12, p1y, p0y)),
    ]
}
