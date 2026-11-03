//! A zoomed pair: a step unit's two versions flying as one box. It is one entry
//! of the plan, so nothing that walks the plan can hold one version without the
//! other, and a flight alone carries no zoom.
use crate::{
    css_layout::paint::Radii,
    live_move::follow::Flight,
    source::math::{mat2d::Mat2D, vec2d::Vec2D},
};

/// What a pair reads once when its step starts, in the page's drawing space.
#[derive(Clone, Copy, Debug)]
pub struct Ends {
    /// The leaving frame as drawn when the step starts.
    pub leaving: Mat2D,
    /// The arriving frame at its resting place.
    pub arriving: Mat2D,
    pub leaving_size: Vec2D,
    pub arriving_size: Vec2D,
    pub leaving_corners: Radii,
    pub arriving_corners: Radii,
}

pub struct Pair {
    flights: [Flight; 2],
    ends: Ends,
}

impl Pair {
    pub(super) fn flights(&self) -> &[Flight] {
        &self.flights
    }
    #[cfg_attr(not(feature = "test-support"), allow(dead_code))]
    pub(super) fn ends(&self) -> Ends {
        self.ends
    }
}
