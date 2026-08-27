use core::ops::{Index, IndexMut, Mul, MulAssign};

use super::aabb::Aabb;
use super::transform_components::TransformComponents;
use super::vec2d::Vec2D;

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
                    self.buffer[0] * point.x + self.buffer[4],
                    self.buffer[3] * point.y + self.buffer[5],
                );
            }
        } else {
            for (out, point) in dst.iter_mut().zip(points) {
                *out = *self * *point;
            }
        }
    }
    pub fn map_bounding_box_points(&self, points: &[Vec2D]) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for point in points {
            let x = self.buffer[0] * point.x + self.buffer[2] * point.y;
            let y = self.buffer[1] * point.x + self.buffer[3] * point.y;
            if !x.is_nan() {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
            if !y.is_nan() {
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        if !((max_x - min_x) >= 0.0 && (max_y - min_y) >= 0.0) {
            return Aabb::default();
        }
        let result = Aabb::new(
            min_x + self.buffer[4],
            min_y + self.buffer[5],
            max_x + self.buffer[4],
            max_y + self.buffer[5],
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
        let mut det = aa * ad - ab * ac;
        if det == 0.0 {
            return false;
        }
        det = 1.0 / det;
        *result = Self::new(
            ad * det,
            -ab * det,
            -ac * det,
            aa * det,
            (ac * aty - ad * atx) * det,
            (ab * atx - aa * aty) * det,
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
            result[2] = result[0] * skew + result[2];
            result[3] = result[1] * skew + result[3];
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
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
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
        self[0] * self[3] - self[2] * self[1]
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
            self[0] * value.x + self[2] * value.y + self[4],
            self[1] * value.x + self[3] * value.y + self[5],
        )
    }
}
