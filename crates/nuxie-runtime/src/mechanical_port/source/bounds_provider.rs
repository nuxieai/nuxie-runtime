use crate::mechanical_port::source::math::{aabb::Aabb, mat2d::Mat2D};

pub trait BoundsProvider {
    fn compute_bounds(&self, to_parent: Mat2D) -> Aabb;
}
