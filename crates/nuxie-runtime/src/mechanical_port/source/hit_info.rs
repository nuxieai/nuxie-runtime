use crate::mechanical_port::source::{artboard::NestedArtboard, math::aabb::IAabb};

pub struct HitInfo {
    pub area: IAabb,
    pub mounts: Vec<*mut NestedArtboard>,
}
