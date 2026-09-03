use core::ops::{Index, IndexMut, Mul, MulAssign};

use super::aabb::Aabb;
use super::transform_components::TransformComponents;
use super::vec2d::Vec2D;

#[inline(always)]
fn simd_min(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() | second.to_bits())
    } else if second < first {
        second
    } else {
        first
    }
}

#[inline(always)]
fn simd_max(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() & second.to_bits())
    } else if first < second {
        second
    } else {
        first
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat2D {
    buffer: [f32; 6],
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat2D {
    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32, tx: f32, ty: f32) -> Self {
        Self {
            buffer: [x1, y1, x2, y2, tx, ty],
        }
    }
    pub fn values(&self) -> &[f32; 6] {
        &self.buffer
    }
    pub fn from_rotation(rad: f32) -> Self {
        let (mut sin, mut cos) = (0.0, 1.0);
        if rad != 0.0 {
            sin = rad.sin();
            cos = rad.cos();
        }
        Self::new(cos, sin, -sin, cos, 0.0, 0.0)
    }
    pub const fn from_scale(sx: f32, sy: f32) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }
    pub const fn from_translate(tx: f32, ty: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, tx, ty)
    }
    pub const fn from_translation(value: Vec2D) -> Self {
        Self::from_translate(value.x, value.y)
    }
    pub const fn from_scale_and_translation(sx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Self::new(sx, 0.0, 0.0, sy, tx, ty)
    }
    pub fn scale_by_values(&mut self, sx: f32, sy: f32) {
        self.buffer[0] *= sx;
        self.buffer[1] *= sx;
        self.buffer[2] *= sy;
        self.buffer[3] *= sy;
    }
    pub fn map_points(&self, dst: &mut [Vec2D], points: &[Vec2D]) {
        assert!(dst.len() >= points.len());
        if self.buffer[1] == 0.0 && self.buffer[2] == 0.0 {
            for (out, point) in dst.iter_mut().zip(points) {
                *out = Vec2D::new(
                    point.x.mul_add(self.buffer[0], self.buffer[4]),
                    point.y.mul_add(self.buffer[3], self.buffer[5]),
                );
            }
        } else {
            for (out, point) in dst.iter_mut().zip(points) {
                let skewed_x = point.y.mul_add(self.buffer[2], self.buffer[4]);
                let skewed_y = point.x.mul_add(self.buffer[1], self.buffer[5]);
                *out = Vec2D::new(
                    point.x.mul_add(self.buffer[0], skewed_x),
                    point.y.mul_add(self.buffer[3], skewed_y),
                );
            }
        }
    }
    pub fn map_bounding_box_points(&self, points: &[Vec2D]) -> Aabb {
        let [scale_x, skew_y, skew_x, scale_y, translate_x, translate_y] = self.buffer;
        let no_skew = skew_y == 0.0 && skew_x == 0.0;
        let mut mins = [f32::INFINITY; 4];
        let mut maxes = [f32::NEG_INFINITY; 4];
        let mut index = 0;

        if points.len() & 1 != 0 {
            let point = points[0];
            let mapped = if no_skew {
                [scale_x * point.x, scale_y * point.y]
            } else {
                [
                    point.x.mul_add(scale_x, skew_x * point.y),
                    point.y.mul_add(scale_y, skew_y * point.x),
                ]
            };
            mins[0] = mapped[0];
            mins[1] = mapped[1];
            maxes[0] = mapped[0];
            maxes[1] = mapped[1];
            index = 1;
        }

        while index < points.len() {
            let first = points[index];
            let second = points[index + 1];
            let mapped = if no_skew {
                [
                    scale_x * first.x,
                    scale_y * first.y,
                    scale_x * second.x,
                    scale_y * second.y,
                ]
            } else {
                [
                    first.x.mul_add(scale_x, skew_x * first.y),
                    first.y.mul_add(scale_y, skew_y * first.x),
                    second.x.mul_add(scale_x, skew_x * second.y),
                    second.y.mul_add(scale_y, skew_y * second.x),
                ]
            };
            for lane in 0..4 {
                mins[lane] = simd_min(mapped[lane], mins[lane]);
                maxes[lane] = simd_max(mapped[lane], maxes[lane]);
            }
            index += 2;
        }

        let min_x = simd_min(mins[0], mins[2]);
        let min_y = simd_min(mins[1], mins[3]);
        let max_x = simd_max(maxes[0], maxes[2]);
        let max_y = simd_max(maxes[1], maxes[3]);
        if !((max_x - min_x) >= 0.0 && (max_y - min_y) >= 0.0) {
            return Aabb::default();
        }
        let result = Aabb::new(
            min_x + translate_x,
            min_y + translate_y,
            max_x + translate_x,
            max_y + translate_y,
        );
        assert!(result.width() >= 0.0);
        assert!(result.height() >= 0.0);
        result
    }
    pub fn map_bounding_box(&self, aabb: Aabb) -> Aabb {
        self.map_bounding_box_points(&[
            Vec2D::new(aabb.left(), aabb.top()),
            Vec2D::new(aabb.right(), aabb.top()),
            Vec2D::new(aabb.right(), aabb.bottom()),
            Vec2D::new(aabb.left(), aabb.bottom()),
        ])
    }
    pub fn invert(&self, result: &mut Self) -> bool {
        let (aa, ab, ac, ad, atx, aty) = (self[0], self[1], self[2], self[3], self[4], self[5]);
        let mut det = aa.mul_add(ad, -(ab * ac));
        if det == 0.0 {
            return false;
        }
        det = 1.0 / det;
        *result = Self::new(
            ad * det,
            -ab * det,
            -ac * det,
            aa * det,
            ac.mul_add(aty, -(ad * atx)) * det,
            ab.mul_add(atx, -(aa * aty)) * det,
        );
        true
    }
    pub fn invert_or_identity(&self) -> Self {
        let mut inverse = Self::identity();
        self.invert(&mut inverse);
        inverse
    }
    pub fn decompose(&self) -> TransformComponents {
        let (m0, m1, m2, m3) = (self[0], self[1], self[2], self[3]);
        let rotation = m1.atan2(m0);
        let denom = m0 * m0 + m1 * m1;
        let scale_x = denom.sqrt();
        let scale_y = if scale_x == 0.0 {
            0.0
        } else {
            (m0 * m3 - m2 * m1) / scale_x
        };
        let skew = (m0 * m2 + m1 * m3).atan2(denom);
        let mut result = TransformComponents::default();
        result.set_x(self[4]);
        result.set_y(self[5]);
        result.set_scale_x(scale_x);
        result.set_scale_y(scale_y);
        result.set_rotation(rotation);
        result.set_skew(skew);
        result
    }
    pub fn compose(components: &TransformComponents) -> Self {
        let mut result = Self::from_rotation(components.rotation());
        result[4] = components.x();
        result[5] = components.y();
        result = result.scale(components.scale());
        let skew = components.skew();
        if skew != 0.0 {
            result[2] = result[0].mul_add(skew, result[2]);
            result[3] = result[1].mul_add(skew, result[3]);
        }
        result
    }
    pub fn scale(self, value: Vec2D) -> Self {
        Self::new(
            self[0] * value.x,
            self[1] * value.x,
            self[2] * value.y,
            self[3] * value.y,
            self[4],
            self[5],
        )
    }
    pub fn translate(self, value: Vec2D) -> Self {
        Self::new(
            self[0],
            self[1],
            self[2],
            self[3],
            self[4] + value.x,
            self[5] + value.y,
        )
    }
    pub fn multiply(a: Self, b: Self) -> Self {
        Self::new(
            a[0].mul_add(b[0], a[2] * b[1]),
            a[1].mul_add(b[0], a[3] * b[1]),
            a[0].mul_add(b[2], a[2] * b[3]),
            a[1].mul_add(b[2], a[3] * b[3]),
            a[0].mul_add(b[4], a[2] * b[5]) + a[4],
            a[1].mul_add(b[4], a[3] * b[5]) + a[5],
        )
    }
    pub fn xx(self) -> f32 {
        self[0]
    }
    pub fn xy(self) -> f32 {
        self[1]
    }
    pub fn yx(self) -> f32 {
        self[2]
    }
    pub fn yy(self) -> f32 {
        self[3]
    }
    pub fn tx(self) -> f32 {
        self[4]
    }
    pub fn ty(self) -> f32 {
        self[5]
    }
    pub fn set_xx(&mut self, value: f32) {
        self[0] = value;
    }
    pub fn set_xy(&mut self, value: f32) {
        self[1] = value;
    }
    pub fn set_yx(&mut self, value: f32) {
        self[2] = value;
    }
    pub fn set_yy(&mut self, value: f32) {
        self[3] = value;
    }
    pub fn set_tx(&mut self, value: f32) {
        self[4] = value;
    }
    pub fn set_ty(&mut self, value: f32) {
        self[5] = value;
    }
    pub fn translation(self) -> Vec2D {
        Vec2D::new(self[4], self[5])
    }
    pub fn determinant(self) -> f32 {
        self[0].mul_add(self[3], -(self[2] * self[1]))
    }
}

impl Index<usize> for Mat2D {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        &self.buffer[index]
    }
}
impl IndexMut<usize> for Mat2D {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.buffer[index]
    }
}
impl Mul for Mat2D {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::multiply(self, rhs)
    }
}
impl MulAssign for Mat2D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self::multiply(*self, rhs);
    }
}
impl Mul<Vec2D> for Mat2D {
    type Output = Vec2D;
    fn mul(self, value: Vec2D) -> Vec2D {
        Vec2D::new(
            self[0].mul_add(value.x, self[2] * value.y) + self[4],
            self[1].mul_add(value.x, self[3] * value.y) + self[5],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiply_matches_pinned_fused_product_sums() {
        let bone_world = Mat2D::new(
            -0.187692404,
            -0.775338828,
            0.771069884,
            -0.205327198,
            284.760468,
            272.738983,
        );
        let inverse_bind = Mat2D::new(
            -0.246825233,
            0.974436581,
            -0.969326734,
            -0.224662051,
            300.711243,
            -294.504639,
        );

        let result = bone_world * inverse_bind;
        assert_eq!(
            result.values().map(f32::to_bits),
            [
                0x3f4c_3525,
                0xbc0e_a00d,
                0x3c0e_9fef,
                0x3f4c_3524,
                0x3f9e_2800,
                0x42c8_1c84,
            ]
        );
    }

    #[test]
    fn map_point_matches_pinned_fused_linear_sum() {
        let world = Mat2D::new(
            0.835797548,
            -0.167389423,
            0.159021586,
            0.884335815,
            352.651398,
            220.692368,
        );

        let result = world * Vec2D::new(43.021419525146484, -749.8175659179688);
        assert_eq!(result.x.to_bits(), 0x4386_af8b);
        assert_eq!(result.y.to_bits(), 0xc3e0_ccbc);
    }

    #[test]
    fn map_points_matches_pinned_simd_fma_stages() {
        let matrix = Mat2D::new(-189.62933, 0.0, 83.74298, 1.0, -162.7352, 0.0);
        let point = Vec2D::new(52.703873, -199.69064);
        let mut result = [Vec2D::default()];

        matrix.map_points(&mut result, &[point]);

        assert_eq!(result[0].x.to_bits(), 0xc6d1_ff41);
        assert_ne!(
            result[0].x.to_bits(),
            (point.x * matrix[0] + (point.y * matrix[2] + matrix[4])).to_bits()
        );
    }

    #[test]
    fn invert_matches_pinned_fused_determinant_and_translation() {
        let world = Mat2D::new(
            -1.5231051445007324,
            0.16598844528198242,
            -0.8321806192398071,
            -0.213197261095047,
            403.1590270996094,
            513.412841796875,
        );
        let mut inverse = Mat2D::identity();

        assert!(world.invert(&mut inverse));
        assert_eq!(
            inverse.values().map(f32::to_bits),
            [
                0xbeeb_d5a2,
                0xbeb7_9cf2,
                0x3fe6_22a6,
                0xc052_9a80,
                0xc438_585e,
                0x44e5_41db,
            ]
        );
    }

    #[test]
    fn decompose_and_compose_match_pinned_source_arithmetic() {
        let world = Mat2D::new(
            f32::from_bits(0xbe42_8f5c),
            f32::from_bits(0xb2f6_7039),
            f32::from_bits(0x32f6_7039),
            f32::from_bits(0xbe42_8f5c),
            f32::from_bits(0x43c6_73b2),
            f32::from_bits(0x43a0_b75d),
        );

        let components = world.decompose();
        assert_eq!(components.x().to_bits(), 0x43c6_73b2);
        assert_eq!(components.y().to_bits(), 0x43a0_b75d);
        assert_eq!(components.scale_x().to_bits(), 0x3e42_8f5c);
        assert_eq!(components.scale_y().to_bits(), 0x3e42_8f5c);
        assert_eq!(components.rotation().to_bits(), 0xc049_0fda);
        // Upstream spells this as ordinary products and a sum. Keeping an
        // explicit fused multiply-add here perturbs repeated IK decomposition
        // enough to fail the upstream 1000-iteration test.
        assert_eq!(components.skew().to_bits(), 0);

        let recomposed = Mat2D::compose(&components);
        assert_eq!(
            recomposed.values().map(f32::to_bits),
            [
                0xbe42_8f5c,
                0xb2f6_7039,
                0x32f6_7039,
                0xbe42_8f5c,
                0x43c6_73b2,
                0x43a0_b75d,
            ]
        );
    }
}
