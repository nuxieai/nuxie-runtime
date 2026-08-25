use crate::Vec2D;

// Direct source-correspondence owner for pinned `include/rive/math/aabb.hpp`
// and `src/math/aabb.cpp`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypedAabb<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

pub type IntegerAabb = TypedAabb<i32>;
pub type AABBi16 = TypedAabb<i16>;
pub type AABBu16 = TypedAabb<u16>;

pub trait AabbScalarBounds: Copy {
    const MIN: Self;
    const MAX: Self;
}

macro_rules! impl_aabb_scalar_bounds {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AabbScalarBounds for $ty {
                const MIN: Self = <$ty>::MIN;
                const MAX: Self = <$ty>::MAX;
            }
        )+
    };
}

impl_aabb_scalar_bounds!(i16, u16, i32, u32, i64, u64);

/// Integer domain used by the pinned `TAABB<T>` aliases and tests.
///
/// All supported values fit in `i128`, which gives the signedness-independent
/// comparison domain used by C++ `math::cmp_*` without converting a negative
/// signed operand to unsigned first.
pub trait AabbInteger: AabbScalarBounds + Copy + Ord {
    const MIN_I128: i128;
    const MAX_I128: i128;

    fn to_i128(self) -> i128;
    fn from_i128(value: i128) -> Option<Self>;
    fn from_i128_clamped(value: i128) -> Self;
    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;
    fn wrapping_neg(self) -> Self;
}

macro_rules! impl_aabb_integer {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AabbInteger for $ty {
                const MIN_I128: i128 = <$ty>::MIN as i128;
                const MAX_I128: i128 = <$ty>::MAX as i128;

                fn to_i128(self) -> i128 {
                    self as i128
                }

                fn from_i128(value: i128) -> Option<Self> {
                    Self::try_from(value).ok()
                }

                fn from_i128_clamped(value: i128) -> Self {
                    value.clamp(Self::MIN_I128, Self::MAX_I128) as Self
                }

                fn wrapping_add(self, other: Self) -> Self {
                    self.wrapping_add(other)
                }

                fn wrapping_sub(self, other: Self) -> Self {
                    self.wrapping_sub(other)
                }

                fn wrapping_neg(self) -> Self {
                    self.wrapping_neg()
                }
            }
        )+
    };
}

impl_aabb_integer!(i16, u16, i32, u32, i64, u64);

impl<T: Copy> TypedAabb<T> {
    pub const fn new(left: T, top: T, right: T, bottom: T) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl<T: AabbInteger> TypedAabb<T> {
    pub fn width(self) -> T {
        self.right.wrapping_sub(self.left)
    }

    pub fn height(self) -> T {
        self.bottom.wrapping_sub(self.top)
    }

    pub fn empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub fn inset(self, dx: T, dy: T) -> Self {
        Self::new(
            self.left.wrapping_add(dx),
            self.top.wrapping_add(dy),
            self.right.wrapping_sub(dx),
            self.bottom.wrapping_sub(dy),
        )
    }

    pub fn outset(self, dx: T, dy: T) -> Self {
        self.inset(dx.wrapping_neg(), dy.wrapping_neg())
    }

    pub fn offset(self, dx: T, dy: T) -> Self {
        Self::new(
            self.left.wrapping_add(dx),
            self.top.wrapping_add(dy),
            self.right.wrapping_add(dx),
            self.bottom.wrapping_add(dy),
        )
    }

    pub fn join(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub fn intersect<U: AabbInteger>(self, other: TypedAabb<U>) -> Self {
        Self {
            left: self.left.max(T::from_i128_clamped(other.left.to_i128())),
            top: self.top.max(T::from_i128_clamped(other.top.to_i128())),
            right: self.right.min(T::from_i128_clamped(other.right.to_i128())),
            bottom: self
                .bottom
                .min(T::from_i128_clamped(other.bottom.to_i128())),
        }
    }

    pub fn intersect_or_empty<U: AabbInteger>(self, other: TypedAabb<U>) -> Self {
        let intersection = self.intersect(other);
        if intersection.empty() {
            Self::new(
                T::from_i128(0).expect("supported AABB integers represent zero"),
                T::from_i128(0).expect("supported AABB integers represent zero"),
                T::from_i128(0).expect("supported AABB integers represent zero"),
                T::from_i128(0).expect("supported AABB integers represent zero"),
            )
        } else {
            intersection
        }
    }

    pub fn lossless_numeric_cast<U: AabbInteger>(self) -> Option<TypedAabb<U>> {
        Some(TypedAabb::new(
            U::from_i128(self.left.to_i128())?,
            U::from_i128(self.top.to_i128())?,
            U::from_i128(self.right.to_i128())?,
            U::from_i128(self.bottom.to_i128())?,
        ))
    }

    pub fn clamp_cast<U: AabbInteger>(self) -> TypedAabb<U> {
        TypedAabb::new(
            U::from_i128_clamped(self.left.to_i128()),
            U::from_i128_clamped(self.top.to_i128()),
            U::from_i128_clamped(self.right.to_i128()),
            U::from_i128_clamped(self.bottom.to_i128()),
        )
    }

    pub fn equals<U: AabbInteger>(self, other: TypedAabb<U>) -> bool {
        self.left.to_i128() == other.left.to_i128()
            && self.right.to_i128() == other.right.to_i128()
            && self.top.to_i128() == other.top.to_i128()
            && self.bottom.to_i128() == other.bottom.to_i128()
    }

    pub fn contains<U: AabbInteger>(self, other: TypedAabb<U>) -> bool {
        self.left.to_i128() <= other.left.to_i128()
            && self.top.to_i128() <= other.top.to_i128()
            && self.right.to_i128() >= other.right.to_i128()
            && self.bottom.to_i128() >= other.bottom.to_i128()
    }

    pub fn overlaps<U: AabbInteger>(self, other: TypedAabb<U>) -> bool {
        self.left.to_i128() < other.right.to_i128()
            && self.right.to_i128() > other.left.to_i128()
            && self.top.to_i128() < other.bottom.to_i128()
            && self.bottom.to_i128() > other.top.to_i128()
    }

    pub fn make_wh<U: AabbInteger>(width: U, height: U) -> Option<Self> {
        let zero = T::from_i128(0).expect("supported AABB integers represent zero");
        Some(Self::new(
            zero,
            zero,
            T::from_i128(width.to_i128())?,
            T::from_i128(height.to_i128())?,
        ))
    }
}

impl<T: AabbScalarBounds> TypedAabb<T> {
    pub const fn make_maximal() -> Self {
        Self::new(T::MIN, T::MIN, T::MAX, T::MAX)
    }

    pub const fn make_maximally_negative() -> Self {
        Self::new(T::MAX, T::MAX, T::MIN, T::MIN)
    }
}

/// Exact shared owner for pinned float `AABB` geometry.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
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

    pub const fn from_ltwh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    pub fn from_integer(bounds: IntegerAabb) -> Self {
        Self::new(
            bounds.left as f32,
            bounds.top as f32,
            bounds.right as f32,
            bounds.bottom as f32,
        )
    }

    pub fn from_points(points: &[Vec2D]) -> Self {
        let Some(first) = points.first() else {
            return Self::default();
        };
        let mut bounds = Self::new(first.x, first.y, first.x, first.y);
        for point in &points[1..] {
            bounds.min_x = cpp_ordered_min(bounds.min_x, point.x);
            bounds.max_x = cpp_ordered_max(bounds.max_x, point.x);
            bounds.min_y = cpp_ordered_min(bounds.min_y, point.y);
            bounds.max_y = cpp_ordered_max(bounds.max_y, point.y);
        }
        bounds
    }

    pub const fn min(self) -> Vec2D {
        Vec2D::new(self.min_x, self.min_y)
    }

    pub const fn max(self) -> Vec2D {
        Vec2D::new(self.max_x, self.max_y)
    }

    pub const fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    pub const fn height(self) -> f32 {
        self.max_y - self.min_y
    }

    pub const fn size(self) -> Vec2D {
        Vec2D::new(self.width(), self.height())
    }

    pub const fn center(self) -> Vec2D {
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
        debug_assert!(result.width() >= 0.0);
        debug_assert!(result.height() >= 0.0);
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

    pub fn round(self) -> IntegerAabb {
        IntegerAabb::new(
            aabb_graphics_round(self.min_x),
            aabb_graphics_round(self.min_y),
            aabb_graphics_round(self.max_x),
            aabb_graphics_round(self.max_y),
        )
    }

    pub fn round_out(self) -> IntegerAabb {
        IntegerAabb::new(
            self.min_x.floor() as i32,
            self.min_y.floor() as i32,
            self.max_x.ceil() as i32,
            self.max_y.ceil() as i32,
        )
    }

    pub const fn for_expansion() -> Self {
        Self::new(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX)
    }

    pub fn expand_to(&mut self, point: Vec2D) {
        self.expand_to_xy(point.x, point.y);
    }

    pub fn expand_to_xy(&mut self, x: f32, y: f32) {
        if x < self.min_x {
            self.min_x = x;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn join(first: Self, second: Self) -> Self {
        Self::new(
            cpp_ordered_min(first.min_x, second.min_x),
            cpp_ordered_min(first.min_y, second.min_y),
            cpp_ordered_max(first.max_x, second.max_x),
            cpp_ordered_max(first.max_y, second.max_y),
        )
    }

    pub fn expand(&mut self, other: Self) {
        *self = Self::join(*self, other);
    }

    pub fn factor_from(self, point: Vec2D) -> Vec2D {
        Vec2D::new(
            if self.width() == 0.0 {
                0.0
            } else {
                (point.x - self.min_x) * 2.0 / self.width() - 1.0
            },
            (if self.height() == 0.0 {
                0.0
            } else {
                point.y - self.min_y
            }) * 2.0
                / self.height()
                - 1.0,
        )
    }

    pub fn contains(self, point: Vec2D) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
    }

    pub fn corner(self, index: usize) -> Option<Vec2D> {
        match index {
            0 => Some(self.min()),
            1 => Some(self.max()),
            _ => None,
        }
    }
}

fn aabb_graphics_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

fn cpp_ordered_min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}

fn cpp_ordered_max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}
