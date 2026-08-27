//! Mechanical translation of pinned
//! `src/constraints/scrolling/clamped_scroll_physics.cpp` and its primary
//! header.

use super::super::*;

impl RuntimeScrollPhysicsState {
    /// The header-owned `m_value` is a default-constructed `Vec2D`.
    pub(crate) fn clamped() -> Self {
        Self {
            kind: crate::components::RuntimeScrollPhysicsKind::Clamped {
                value: (0.0, 0.0),
            },
            last_time_micros: 0,
            is_running: false,
            speed: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            direction: 1,
            friction: 8.0,
            speed_multiplier: 1.0,
            elastic_factor: 0.66,
        }
    }

    pub(in crate::constraints) fn clamped_advance(
        &mut self,
        _elapsed_seconds: f32,
    ) -> (f32, f32) {
        self.stop();
        match self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value } => value,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn clamped_run(
        &mut self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
        snapping_points: &[(f32, f32)],
        content_size: f32,
        viewport_size: f32,
    ) {
        self.scroll_physics_run(
            range_min,
            range_max,
            value,
            snapping_points,
            content_size,
            viewport_size,
        );
        let clamped = self.clamped_clamp(range_min, range_max, value);
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value } => *value = clamped,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn clamped_clamp(
        &self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
    ) -> (f32, f32) {
        (
            rive_math_clamp(value.0, range_min.0, range_max.0),
            rive_math_clamp(value.1, range_min.1, range_max.1),
        )
    }
}
