use crate::mechanical_port::source::{
    constraints::scrolling::scroll_physics::{ScrollPhysics, ScrollPhysicsRuntime},
    generated::constraints::scrolling::clamped_scroll_physics_base::ClampedScrollPhysicsBase,
    math::{math_types, vec2d::Vec2D},
};

pub struct ClampedScrollPhysics {
    pub base: ClampedScrollPhysicsBase,
    value: Vec2D,
}

impl Default for ClampedScrollPhysics {
    fn default() -> Self {
        Self {
            base: ClampedScrollPhysicsBase::default(),
            value: Vec2D::default(),
        }
    }
}

impl ClampedScrollPhysics {
    pub fn advance(&mut self, _elapsed_seconds: f32) -> Vec2D {
        ScrollPhysicsRuntime::stop(self);
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
        self.base.base.run_base();
        self.value = self.clamp(range_min, range_max, value);
    }

    pub fn clamp(&self, range_min: Vec2D, range_max: Vec2D, value: Vec2D) -> Vec2D {
        Vec2D::new(
            math_types::clamp(value.x, range_min.x, range_max.x),
            math_types::clamp(value.y, range_min.y, range_max.y),
        )
    }
}

impl ScrollPhysicsRuntime for ClampedScrollPhysics {
    fn physics(&self) -> &ScrollPhysics {
        &self.base.base
    }

    fn physics_mut(&mut self) -> &mut ScrollPhysics {
        &mut self.base.base
    }

    fn advance(&mut self, elapsed_seconds: f32) -> Vec2D {
        ClampedScrollPhysics::advance(self, elapsed_seconds)
    }

    fn run(
        &mut self,
        range_min: Vec2D,
        range_max: Vec2D,
        value: Vec2D,
        snapping_points: Vec<Vec2D>,
        content_size: f32,
        viewport_size: f32,
    ) {
        ClampedScrollPhysics::run(
            self,
            range_min,
            range_max,
            value,
            snapping_points,
            content_size,
            viewport_size,
        );
    }

    fn clamp(&self, range_min: Vec2D, range_max: Vec2D, value: Vec2D) -> Vec2D {
        ClampedScrollPhysics::clamp(self, range_min, range_max, value)
    }
}
