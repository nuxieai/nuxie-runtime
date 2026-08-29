use core::ops::{Add, Neg, Sub};

use super::{math_types, vec2d::Vec2D};

/// Rust dispatch for the signedness-aware comparisons used by C++ TAABB.
/// Integer operands go through the same helpers as the other math owners;
/// same-type floats retain their ordinary partial-order behavior.
pub trait BoundsComparison<Rhs = Self> {
    fn bounds_equal(self, rhs: Rhs) -> bool;
    fn bounds_less(self, rhs: Rhs) -> bool;
    fn bounds_less_equal(self, rhs: Rhs) -> bool;
    fn bounds_greater(self, rhs: Rhs) -> bool;
    fn bounds_greater_equal(self, rhs: Rhs) -> bool;
}

macro_rules! integer_bounds_comparison {
    ($($ty:ty),* $(,)?) => {$(
        impl<U: math_types::Integer> BoundsComparison<U> for $ty {
            fn bounds_equal(self, rhs: U) -> bool { math_types::cmp_equal(self, rhs) }
            fn bounds_less(self, rhs: U) -> bool { math_types::cmp_less(self, rhs) }
            fn bounds_less_equal(self, rhs: U) -> bool { math_types::cmp_less_equal(self, rhs) }
            fn bounds_greater(self, rhs: U) -> bool { math_types::cmp_greater(self, rhs) }
            fn bounds_greater_equal(self, rhs: U) -> bool { math_types::cmp_greater_equal(self, rhs) }
        }
    )*};
}
integer_bounds_comparison!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

macro_rules! float_bounds_comparison {
    ($($ty:ty),* $(,)?) => {$(
        impl BoundsComparison for $ty {
            fn bounds_equal(self, rhs: Self) -> bool { self == rhs }
            fn bounds_less(self, rhs: Self) -> bool { self < rhs }
            fn bounds_less_equal(self, rhs: Self) -> bool { self <= rhs }
            fn bounds_greater(self, rhs: Self) -> bool { self > rhs }
            fn bounds_greater_equal(self, rhs: Self) -> bool { self >= rhs }
        }
    )*};
}
float_bounds_comparison!(f32, f64);

pub trait NumericBounds: Copy + PartialOrd {
    fn min_value() -> Self;
    fn max_value() -> Self;
    fn zero() -> Self;
}

macro_rules! numeric_bounds {
    ($($ty:ty),* $(,)?) => {$(
        impl NumericBounds for $ty {
            fn min_value() -> Self { <$ty>::MIN }
            fn max_value() -> Self { <$ty>::MAX }
            fn zero() -> Self { 0 }
        }
    )*};
}
numeric_bounds!(i16, u16, i32, u32, i64, u64, isize, usize);

pub trait ClampFrom<U> {
    fn clamp_from(value: U) -> Self;
}
pub trait LosslessFrom<U> {
    fn lossless_from(value: U) -> Self;
}

macro_rules! integer_casts {
    ($dst:ty; $($src:ty),* $(,)?) => {$(
        impl ClampFrom<$src> for $dst {
            fn clamp_from(value: $src) -> Self {
                (value as i128).clamp(<$dst>::MIN as i128, <$dst>::MAX as i128) as $dst
            }
        }
        impl LosslessFrom<$src> for $dst {
            fn lossless_from(value: $src) -> Self {
                let result = value as $dst;
                assert_eq!(result as i128, value as i128);
                result
            }
        }
    )*};
}
integer_casts!(i16; i16, u16, i32, u32, i64, u64, isize, usize);
integer_casts!(u16; i16, u16, i32, u32, i64, u64, isize, usize);
integer_casts!(i32; i16, u16, i32, u32, i64, u64, isize, usize);
integer_casts!(u32; i16, u16, i32, u32, i64, u64, isize, usize);

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct TAabb<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T: Copy + BoundsComparison<U>, U: Copy> PartialEq<TAabb<U>> for TAabb<T> {
    fn eq(&self, other: &TAabb<U>) -> bool {
        self.left.bounds_equal(other.left)
            && self.right.bounds_equal(other.right)
            && self.top.bounds_equal(other.top)
            && self.bottom.bounds_equal(other.bottom)
    }
}

impl<T: Copy + Eq + BoundsComparison> Eq for TAabb<T> {}

impl<T> TAabb<T>
where
    T: Copy + PartialOrd + Sub<Output = T>,
{
    pub fn width(self) -> T {
        self.right - self.left
    }
    pub fn height(self) -> T {
        self.bottom - self.top
    }
    pub fn empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
}

impl<T: NumericBounds> TAabb<T> {
    pub fn make_maximal() -> Self {
        Self {
            left: T::min_value(),
            top: T::min_value(),
            right: T::max_value(),
            bottom: T::max_value(),
        }
    }
    pub fn make_maximally_negative() -> Self {
        Self {
            left: T::max_value(),
            top: T::max_value(),
            right: T::min_value(),
            bottom: T::min_value(),
        }
    }
    pub fn make_wh<U>(width: U, height: U) -> Self
    where
        T: LosslessFrom<U>,
        U: Copy,
    {
        Self {
            left: T::zero(),
            top: T::zero(),
            right: T::lossless_from(width),
            bottom: T::lossless_from(height),
        }
    }
}

impl<T> TAabb<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T>,
{
    pub fn inset(self, dx: T, dy: T) -> Self {
        Self {
            left: self.left + dx,
            top: self.top + dy,
            right: self.right - dx,
            bottom: self.bottom - dy,
        }
    }
    pub fn offset(self, dx: T, dy: T) -> Self {
        Self {
            left: self.left + dx,
            top: self.top + dy,
            right: self.right + dx,
            bottom: self.bottom + dy,
        }
    }
}

impl<T> TAabb<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Neg<Output = T>,
{
    pub fn outset(self, dx: T, dy: T) -> Self {
        self.inset(-dx, -dy)
    }
}

impl<T: Copy + PartialOrd> TAabb<T> {
    pub fn join(self, b: Self) -> Self {
        Self {
            left: cpp_min(self.left, b.left),
            top: cpp_min(self.top, b.top),
            right: cpp_max(self.right, b.right),
            bottom: cpp_max(self.bottom, b.bottom),
        }
    }
    pub fn contains<U>(self, rhs: TAabb<U>) -> bool
    where
        T: BoundsComparison<U>,
        U: Copy,
    {
        self.left.bounds_less_equal(rhs.left)
            && self.top.bounds_less_equal(rhs.top)
            && self.right.bounds_greater_equal(rhs.right)
            && self.bottom.bounds_greater_equal(rhs.bottom)
    }
    pub fn overlaps<U>(self, b: TAabb<U>) -> bool
    where
        T: BoundsComparison<U>,
        U: Copy,
    {
        self.left.bounds_less(b.right)
            && self.right.bounds_greater(b.left)
            && self.top.bounds_less(b.bottom)
            && self.bottom.bounds_greater(b.top)
    }
    pub fn intersect<U>(self, b: TAabb<U>) -> Self
    where
        T: ClampFrom<U>,
        U: Copy,
    {
        Self {
            left: cpp_max(self.left, T::clamp_from(b.left)),
            top: cpp_max(self.top, T::clamp_from(b.top)),
            right: cpp_min(self.right, T::clamp_from(b.right)),
            bottom: cpp_min(self.bottom, T::clamp_from(b.bottom)),
        }
    }
    pub fn clamp_cast<U>(self) -> TAabb<U>
    where
        U: ClampFrom<T>,
    {
        TAabb {
            left: U::clamp_from(self.left),
            top: U::clamp_from(self.top),
            right: U::clamp_from(self.right),
            bottom: U::clamp_from(self.bottom),
        }
    }
    pub fn lossless_numeric_cast<U>(self) -> TAabb<U>
    where
        U: LosslessFrom<T>,
    {
        TAabb {
            left: U::lossless_from(self.left),
            top: U::lossless_from(self.top),
            right: U::lossless_from(self.right),
            bottom: U::lossless_from(self.bottom),
        }
    }
}

impl<T> TAabb<T>
where
    T: NumericBounds + Copy + PartialOrd + Sub<Output = T>,
{
    pub fn intersect_or_empty<U>(self, b: TAabb<U>) -> Self
    where
        T: ClampFrom<U>,
        U: Copy,
    {
        let result = self.intersect(b);
        if result.empty() {
            Self::default_zero()
        } else {
            result
        }
    }
    fn default_zero() -> Self {
        Self {
            left: T::zero(),
            top: T::zero(),
            right: T::zero(),
            bottom: T::zero(),
        }
    }
}

pub type IAabb = TAabb<i32>;
pub type AabbI16 = TAabb<i16>;
pub type AabbU16 = TAabb<u16>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Aabb {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Aabb {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
    pub const fn from_min_max(min: Vec2D, max: Vec2D) -> Self {
        Self::new(min.x, min.y, max.x, max.y)
    }
    pub fn from_ltwh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }
    pub fn from_iaabb(value: IAabb) -> Self {
        Self::new(
            value.left as f32,
            value.top as f32,
            value.right as f32,
            value.bottom as f32,
        )
    }
    pub fn from_points(points: &[Vec2D]) -> Self {
        let Some(first) = points.first().copied() else {
            return Self::default();
        };
        let (mut left, mut right, mut top, mut bottom) = (first.x, first.x, first.y, first.y);
        for point in &points[1..] {
            left = cpp_min(left, point.x);
            right = cpp_max(right, point.x);
            top = cpp_min(top, point.y);
            bottom = cpp_max(bottom, point.y);
        }
        Self::new(left, top, right, bottom)
    }
    pub fn left(self) -> f32 {
        self.min_x
    }
    pub fn top(self) -> f32 {
        self.min_y
    }
    pub fn right(self) -> f32 {
        self.max_x
    }
    pub fn bottom(self) -> f32 {
        self.max_y
    }
    pub fn min(self) -> Vec2D {
        Vec2D::new(self.min_x, self.min_y)
    }
    pub fn max(self) -> Vec2D {
        Vec2D::new(self.max_x, self.max_y)
    }
    pub fn width(self) -> f32 {
        self.max_x - self.min_x
    }
    pub fn height(self) -> f32 {
        self.max_y - self.min_y
    }
    pub fn size(self) -> Vec2D {
        Vec2D::new(self.width(), self.height())
    }
    pub fn center(self) -> Vec2D {
        Vec2D::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }
    pub fn is_empty_or_nan(self) -> bool {
        !(self.width() > 0.0 && self.height() > 0.0)
    }
    pub fn pad(self, amount: f32) -> Self {
        self.outset(amount, amount)
    }
    pub fn inset(self, dx: f32, dy: f32) -> Self {
        let result = Self::new(
            self.min_x + dx,
            self.min_y + dy,
            self.max_x - dx,
            self.max_y - dy,
        );
        assert!(result.width() >= 0.0);
        assert!(result.height() >= 0.0);
        result
    }
    pub fn outset(self, dx: f32, dy: f32) -> Self {
        self.inset(-dx, -dy)
    }
    pub fn offset(self, dx: f32, dy: f32) -> Self {
        Self::new(
            self.min_x + dx,
            self.min_y + dy,
            self.max_x + dx,
            self.max_y + dy,
        )
    }
    pub fn round(self) -> IAabb {
        TAabb {
            left: graphics_round(self.left()),
            top: graphics_round(self.top()),
            right: graphics_round(self.right()),
            bottom: graphics_round(self.bottom()),
        }
    }
    pub fn round_out(self) -> IAabb {
        TAabb {
            left: self.left().floor() as i32,
            top: self.top().floor() as i32,
            right: self.right().ceil() as i32,
            bottom: self.bottom().ceil() as i32,
        }
    }
    pub fn for_expansion() -> Self {
        Self::new(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX)
    }
    pub fn expand_to_point(out: &mut Self, point: Vec2D) {
        Self::expand_to(out, point.x, point.y);
    }
    pub fn expand_to(out: &mut Self, x: f32, y: f32) {
        if x < out.min_x {
            out.min_x = x;
        }
        if x > out.max_x {
            out.max_x = x;
        }
        if y < out.min_y {
            out.min_y = y;
        }
        if y > out.max_y {
            out.max_y = y;
        }
    }
    pub fn join(out: &mut Self, a: Self, b: Self) {
        *out = Self::new(
            cpp_min(a.min_x, b.min_x),
            cpp_min(a.min_y, b.min_y),
            cpp_max(a.max_x, b.max_x),
            cpp_max(a.max_y, b.max_y),
        );
    }
    pub fn expand(&mut self, other: Self) {
        let current = *self;
        Self::join(self, current, other);
    }
    pub fn factor_from(self, point: Vec2D) -> Vec2D {
        Vec2D::new(
            if self.width() == 0.0 {
                0.0
            } else {
                (point.x - self.left()) * 2.0 / self.width() - 1.0
            },
            (if self.height() == 0.0 {
                0.0
            } else {
                point.y - self.top()
            }) * 2.0
                / self.height()
                - 1.0,
        )
    }
    pub fn contains(self, point: Vec2D) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }
    pub fn overlaps(self, b: Self) -> bool {
        self.min_x < b.max_x && self.max_x > b.min_x && self.min_y < b.max_y && self.max_y > b.min_y
    }
    pub fn corner(self, index: usize) -> Vec2D {
        match index {
            0 => self.min(),
            1 => self.max(),
            _ => unreachable!(),
        }
    }
}

fn graphics_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}
fn cpp_min<T: Copy + PartialOrd>(a: T, b: T) -> T {
    if b < a { b } else { a }
}
fn cpp_max<T: Copy + PartialOrd>(a: T, b: T) -> T {
    if a < b { b } else { a }
}
