use crate::mechanical_port::source::{core::CoreHandle, math::aabb::IAabb};

pub struct HitInfo {
    pub area: IAabb,
    pub mounts: Vec<CoreHandle>,
}
