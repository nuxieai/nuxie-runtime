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
        self.x * self.x + self.y * self.y
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
        a + (b - a) * t
    }

    pub fn transform_dir(a: Self, m: &Mat2D) -> Self {
        Self {
            x: m[0] * a.x + m[2] * a.y,
            y: m[1] * a.x + m[3] * a.y,
        }
    }

    pub fn transform_mat2d(a: Self, m: &Mat2D) -> Self {
        Self {
            x: m[0] * a.x + m[2] * a.y + m[4],
            y: m[1] * a.x + m[3] * a.y + m[5],
        }
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y
    }

    pub fn cross(a: Self, b: Self) -> f32 {
        a.x * b.y - a.y * b.x
    }

    pub fn scale_and_add(a: Self, b: Self, scale: f32) -> Self {
        Self {
            x: a.x + b.x * scale,
            y: a.y + b.y * scale,
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
