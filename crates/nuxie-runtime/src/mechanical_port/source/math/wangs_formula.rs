use super::mat2d::Mat2D;
use super::simd::{self, Float2, Float4, GVec};
use super::vec2d::Vec2D;

pub const fn length_term<const DEGREE: i32>(precision: f32) -> f32 {
    (DEGREE * (DEGREE - 1)) as f32 / 8.0 * precision
}

pub const fn length_term_pow2<const DEGREE: i32>(precision: f32) -> f32 {
    ((DEGREE * DEGREE) * ((DEGREE - 1) * (DEGREE - 1))) as f32 / 64.0 * (precision * precision)
}

pub fn root4(x: f32) -> f32 {
    x.sqrt().sqrt()
}

pub fn sk_float_nextlog2(x: f32) -> i32 {
    let bits = x.to_bits().wrapping_add((1_u32 << 23) - 1);
    let exp = ((bits as i32) >> 23) - 127;
    exp & !(exp >> 31)
}

pub fn nextlog4(x: f32) -> i32 {
    (sk_float_nextlog2(x) + 1) >> 1
}

pub fn nextlog16(x: f32) -> i32 {
    (sk_float_nextlog2(x) + 3) >> 2
}

#[derive(Clone, Copy, Debug)]
#[repr(C, align(32))]
pub struct VectorXform {
    scale: Float4,
    skew: Float4,
}

impl Default for VectorXform {
    fn default() -> Self {
        Self {
            scale: GVec::splat(1.0),
            skew: GVec::splat(0.0),
        }
    }
}

impl VectorXform {
    pub fn from_mat2d(matrix: &Mat2D) -> Self {
        Self {
            scale: GVec::from_array([matrix[0], matrix[3], matrix[0], matrix[3]]),
            skew: GVec::from_array([matrix[2], matrix[1], matrix[2], matrix[1]]),
        }
    }

    pub fn transform2(self, vector: Float2) -> Float2 {
        simd::mul_add(self.scale.xy(), vector, self.skew.xy() * vector.yx())
    }

    pub fn transform4(self, vectors: Float4) -> Float4 {
        simd::mul_add(self.scale, vectors, self.skew * vectors.yxwz())
    }
}

fn load_point(point: Vec2D) -> Float2 {
    GVec::from_array([point.x, point.y])
}

fn std_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

fn std_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

pub fn quadratic_pow4_points(
    p0: Float2,
    p1: Float2,
    p2: Float2,
    precision: f32,
    vector_xform: VectorXform,
) -> f32 {
    let mut v = simd::mul_add(p1, GVec::splat(-2.0), p0) + p2;
    v = vector_xform.transform2(v);
    let vv = v * v;
    (vv[0] + vv[1]) * length_term_pow2::<2>(precision)
}

pub fn quadratic_pow4(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> f32 {
    quadratic_pow4_points(
        load_point(pts[0]),
        load_point(pts[1]),
        load_point(pts[2]),
        precision,
        vector_xform,
    )
}

pub fn quadratic(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> f32 {
    root4(quadratic_pow4(pts, precision, vector_xform))
}

pub fn quadratic_log2(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> i32 {
    nextlog16(quadratic_pow4(pts, precision, vector_xform))
}

pub fn cubic_pow4(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> f32 {
    let p01 = GVec::from_array([pts[0].x, pts[0].y, pts[1].x, pts[1].y]);
    let p12 = GVec::from_array([pts[1].x, pts[1].y, pts[2].x, pts[2].y]);
    let p23 = GVec::from_array([pts[2].x, pts[2].y, pts[3].x, pts[3].y]);
    let mut v = simd::mul_add(p12, GVec::splat(-2.0), p01) + p23;
    v = vector_xform.transform4(v);
    let vv = v * v;
    std_max(vv[0] + vv[1], vv[2] + vv[3]) * length_term_pow2::<3>(precision)
}

pub fn cubic(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> f32 {
    root4(cubic_pow4(pts, precision, vector_xform))
}

pub fn cubic_log2(pts: &[Vec2D], precision: f32, vector_xform: VectorXform) -> i32 {
    nextlog16(cubic_pow4(pts, precision, vector_xform))
}

pub fn worst_case_cubic_pow4(dev_width: f32, dev_height: f32, precision: f32) -> f32 {
    let kk = length_term_pow2::<3>(precision);
    4.0 * kk * (dev_width * dev_width + dev_height * dev_height)
}

pub fn worst_case_cubic(dev_width: f32, dev_height: f32, precision: f32) -> f32 {
    root4(worst_case_cubic_pow4(dev_width, dev_height, precision))
}

pub fn worst_case_cubic_log2(dev_width: f32, dev_height: f32, precision: f32) -> i32 {
    nextlog16(worst_case_cubic_pow4(dev_width, dev_height, precision))
}

pub fn conic_pow2_points(
    precision: f32,
    mut p0: Float2,
    mut p1: Float2,
    mut p2: Float2,
    w: f32,
    vector_xform: VectorXform,
) -> f32 {
    p0 = vector_xform.transform2(p0);
    p1 = vector_xform.transform2(p1);
    p2 = vector_xform.transform2(p2);

    let center = (simd::min(simd::min(p0, p1), p2) + simd::max(simd::max(p0, p1), p2)) * 0.5;
    p0 -= center;
    p1 -= center;
    p2 -= center;

    let max_len = std_max(
        simd::dot(p0, p0),
        std_max(simd::dot(p1, p1), simd::dot(p2, p2)),
    )
    .sqrt();
    let dp = simd::mul_add(p1, GVec::splat(-2.0 * w), p0) + p2;
    let dw = (-2.0 * w + 2.0).abs();
    let rp_minus_1 = std_max(0.0, max_len.mul_add(precision, -1.0));
    let numer = simd::dot(dp, dp).sqrt().mul_add(precision, rp_minus_1 * dw);
    let denom = 4.0 * std_min(w, 1.0);
    numer / denom
}

pub fn conic_pow2(pts: &[Vec2D], precision: f32, w: f32, vector_xform: VectorXform) -> f32 {
    conic_pow2_points(
        precision,
        load_point(pts[0]),
        load_point(pts[1]),
        load_point(pts[2]),
        w,
        vector_xform,
    )
}

pub fn conic(pts: &[Vec2D], tolerance: f32, w: f32, vector_xform: VectorXform) -> f32 {
    conic_pow2(pts, tolerance, w, vector_xform).sqrt()
}

pub fn conic_log2(pts: &[Vec2D], tolerance: f32, w: f32, vector_xform: VectorXform) -> i32 {
    nextlog4(conic_pow2(pts, tolerance, w, vector_xform))
}
