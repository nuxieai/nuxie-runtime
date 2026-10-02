//! Shared scalar translation of rive/shapes/paint/paint_outset.{hpp,cpp}.
use crate::{StrokeCap, StrokeJoin, StrokeParams};

pub const K_MITER_LIMIT: f32 = 4.0;
pub const K_GAUSSIAN_INTEGRAL_STD_DEVS: f32 = 3.0;

pub const fn feather_radius_from_feather(feather: f32) -> f32 {
    feather * (K_GAUSSIAN_INTEGRAL_STD_DEVS / 2.0)
}

pub fn paint_bounds_outset(stroke: Option<&StrokeParams>, feather: f32) -> f32 {
    let mut outset = 0.0;
    if let Some(stroke) = stroke {
        outset = stroke.thickness * 0.5;
        if stroke.join == StrokeJoin::Miter {
            outset *= K_MITER_LIMIT;
        } else if stroke.cap == StrokeCap::Square {
            outset *= std::f32::consts::SQRT_2;
        }
    }
    if feather != 0.0 {
        outset += feather_radius_from_feather(feather);
    }
    outset
}
