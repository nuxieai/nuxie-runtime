use crate::mechanical_port::source::{
    constraints::scrolling::scroll_physics::{ScrollPhysics, ScrollPhysicsState},
    generated::constraints::scrolling::clamped_scroll_physics_base::ClampedScrollPhysicsBase,
    math::vec2d::Vec2D,
};

pub struct ClampedScrollPhysics {
    pub base: ClampedScrollPhysicsBase,
    pub physics: ScrollPhysicsState,
    value: Vec2D,
}

impl ClampedScrollPhysics {
    pub fn advance(&mut self, _elapsed_seconds: f32) -> Vec2D {
        self.stop();
        self.value
    }

    pub fn run(
        &mut self,
        range_min: Vec2D,
        range_max: Vec2D,
        value: Vec2D,
        snapping_points: Vec<Vec2D>,
        content_size: f32,
        viewport_size: f32,
    ) {
        ScrollPhysics::run(
            self,
            range_min,
            range_max,
            value,
            snapping_points,
            content_size,
            viewport_size,
        );
        self.value = self.clamp(range_min, range_max, value);
    }

    pub fn clamp(&self, range_min: Vec2D, range_max: Vec2D, value: Vec2D) -> Vec2D {
        Vec2D::new(
            value.x.clamp(range_min.x, range_max.x),
            value.y.clamp(range_min.y, range_max.y),
        )
    }
}
