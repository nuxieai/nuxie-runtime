// Direct source-correspondence owner for pinned `src/math/aabb.cpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedAabb<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

pub type IntegerAabb = TypedAabb<i32>;

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

impl<T: Copy + Ord> TypedAabb<T> {
    pub fn join(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    pub fn empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
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

#[derive(Clone, Copy)]
struct RuntimeAabb {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl RuntimeAabb {
    fn from_artboard(instance: &ArtboardInstance) -> Self {
        Self {
            left: -instance.width * instance.origin_x,
            top: -instance.height * instance.origin_y,
            width: instance.width,
            height: instance.height,
        }
    }

    fn from_artboard_with_layout(instance: &ArtboardInstance, graph: &ArtboardGraph) -> Self {
        instance
            .retained_layout_bounds()
            .and_then(|bounds| bounds.get(&0).copied())
            .or_else(|| instance.runtime_root_artboard_layout_bounds(graph))
            .map(|bounds| Self {
                left: 0.0,
                top: 0.0,
                width: bounds.width,
                height: bounds.height,
            })
            .unwrap_or_else(|| Self::from_artboard(instance))
    }

    fn from_local_layout_bounds(bounds: RuntimeLayoutBounds) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: bounds.width,
            height: bounds.height,
        }
    }
}
