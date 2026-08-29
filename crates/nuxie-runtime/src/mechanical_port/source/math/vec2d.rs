use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

use super::mat2d::Mat2D;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length_squared(self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let len2 = self.length_squared();
        let scale = if len2 > 0.0 { 1.0 / len2.sqrt() } else { 1.0 };
        self * scale
    }

    pub fn normalize_length(&mut self) -> f32 {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
        }
        len
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self::new((b.x - a.x).mul_add(t, a.x), (b.y - a.y).mul_add(t, a.y))
    }

    pub fn transform_dir(a: Self, m: &Mat2D) -> Self {
        Self {
            x: m[0].mul_add(a.x, m[2] * a.y),
            y: m[1].mul_add(a.x, m[3] * a.y),
        }
    }

    pub fn transform_mat2d(a: Self, m: &Mat2D) -> Self {
        Self {
            x: m[0].mul_add(a.x, m[2] * a.y) + m[4],
            y: m[1].mul_add(a.x, m[3] * a.y) + m[5],
        }
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        a.x.mul_add(b.x, a.y * b.y)
    }

    pub fn cross(a: Self, b: Self) -> f32 {
        a.x.mul_add(b.y, -(a.y * b.x))
    }

    pub fn scale_and_add(a: Self, b: Self, scale: f32) -> Self {
        Self {
            x: b.x.mul_add(scale, a.x),
            y: b.y.mul_add(scale, a.y),
        }
    }

    pub fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }

    pub fn distance_squared(a: Self, b: Self) -> f32 {
        (a - b).length_squared()
    }
}

impl Neg for Vec2D {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}
impl Add for Vec2D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Vec2D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f32> for Vec2D {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
impl Mul<Vec2D> for f32 {
    type Output = Vec2D;
    fn mul(self, rhs: Vec2D) -> Vec2D {
        rhs * self
    }
}
impl Div<f32> for Vec2D {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
impl AddAssign for Vec2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl SubAssign for Vec2D {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl MulAssign<f32> for Vec2D {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
impl DivAssign<f32> for Vec2D {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
impl Index<usize> for Vec2D {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => unreachable!(),
        }
    }
}

impl Eq for Vec2D {}
impl Hash for Vec2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let x = if self.x == 0.0 { 0 } else { self.x.to_bits() } as usize;
        let y = if self.y == 0.0 { 0 } else { self.y.to_bits() } as usize;
        (x ^ y.wrapping_shl(1)).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_squared_matches_pinned_fused_product_sum() {
        let point = Vec2D::new(96.2807, 71.687096);

        assert_eq!(point.length_squared().to_bits(), 0x4661_240d);
        assert_ne!(
            point.length_squared().to_bits(),
            (point.x * point.x + point.y * point.y).to_bits()
        );
    }

    #[test]
    fn scale_and_add_matches_pinned_fused_rounding() {
        let position = Vec2D::new(-0.914693355, -65.6048279);
        let direction = Vec2D::new(-0.215356767, 0.976535439);

        let result = Vec2D::scale_and_add(position, direction, 15.8580017);

        assert_eq!(result.x.to_bits(), 0xc08a_8de5);
        assert_eq!(result.y.to_bits(), 0xc248_79c8);
        assert_ne!(
            result.x.to_bits(),
            (position.x + direction.x * 15.8580017).to_bits()
        );
    }

    #[test]
    fn dot_matches_pinned_fused_product_sum() {
        let to_previous = Vec2D::new(-0.998617827, 0.0525590144);
        let to_next = Vec2D::new(-0.694725275, 0.719275176);

        let result = Vec2D::dot(to_previous, to_next);

        assert_eq!(result.to_bits(), 0x3f3b_4823);
        assert_ne!(
            result.to_bits(),
            (to_previous.x * to_next.x + to_previous.y * to_next.y).to_bits()
        );
    }

    #[test]
    fn cross_matches_pinned_fused_product_difference() {
        let a = Vec2D::new(-191.26535, -64.03908);
        let b = Vec2D::new(-9.758596, -185.07193);

        let result = Vec2D::cross(a, b);

        assert_eq!(result.to_bits(), 0x4707_d4ea);
        assert_ne!(result.to_bits(), (a.x * b.y - a.y * b.x).to_bits());
    }
}
