//! Direct owner for pinned `src/constraints/scrolling/scroll_physics.cpp` and
//! dispatcher for its concrete physics variants.

use super::super::*;

pub(super) fn high_resolution_clock_micros() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_micros());
    i64::try_from(micros).unwrap_or(i64::MAX)
}

impl RuntimeScrollPhysicsState {
    pub(in crate::constraints) fn enabled(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => self.elastic_enabled(),
        }
    }

    pub(in crate::constraints) fn is_running(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => {
                self.elastic_is_running()
            }
        }
    }

    pub(in crate::constraints) fn stop(&mut self) {
        self.is_running = false;
        self.speed = (0.0, 0.0);
    }

    pub(in crate::constraints) fn reset(&mut self) {
        if matches!(
            &self.kind,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. }
        ) {
            self.elastic_reset();
            return;
        }
        self.scroll_physics_reset();
    }

    pub(in crate::constraints) fn scroll_physics_reset(&mut self) {
        self.last_time_micros = if crate::math::random::runtime_deterministic_mode() {
            0
        } else {
            high_resolution_clock_micros()
        };
        self.speed = (0.0, 0.0);
        self.acceleration = (0.0, 0.0);
        self.stop();
    }

    pub(in crate::constraints) fn prepare(&mut self, direction: u64) {
        if matches!(
            &self.kind,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. }
        ) {
            self.elastic_prepare(direction);
            return;
        }
        self.scroll_physics_prepare(direction);
    }

    pub(in crate::constraints) fn scroll_physics_prepare(&mut self, direction: u64) {
        self.reset();
        self.direction = direction;
    }

    pub(in crate::constraints) fn clear_velocity(&mut self) {
        self.speed = (0.0, 0.0);
    }

    pub(in crate::constraints) fn target(&self, axis: RuntimeScrollAxis) -> Option<f32> {
        if !self.is_running() {
            return None;
        }
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => self
                .elastic_has_target(axis)
                .then(|| self.elastic_target_value(axis)),
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => None,
        }
    }

    pub(in crate::constraints) fn scroll_to_position(
        &mut self,
        current: (f32, f32),
        target: (f32, f32),
        range_min: (f32, f32),
        range_max: (f32, f32),
        horizontal: bool,
        vertical: bool,
    ) {
        if matches!(
            &self.kind,
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. }
        ) {
            self.elastic_scroll_to_position(
                current, target, range_min, range_max, horizontal, vertical,
            );
        }
        // Pinned base ScrollPhysics and ClampedScrollPhysics do not override
        // scrollToPosition.
    }

    pub(in crate::constraints) fn accumulate(&mut self, delta: (f32, f32), timestamp: f32) {
        let elapsed_seconds = if crate::math::random::runtime_deterministic_mode() {
            let elapsed = timestamp - self.last_time_micros as f32;
            self.last_time_micros = timestamp as i64;
            elapsed
        } else {
            let now = high_resolution_clock_micros();
            let elapsed = now.saturating_sub(self.last_time_micros) as f32 / 1_000_000.0;
            self.last_time_micros = now;
            elapsed
        };
        if elapsed_seconds > 0.0 {
            let last_speed = self.speed;
            self.speed = (delta.0 / elapsed_seconds, delta.1 / elapsed_seconds);
            self.acceleration = (
                (last_speed.0 + self.speed.0) / elapsed_seconds,
                (last_speed.1 + self.speed.1) / elapsed_seconds,
            );
        }
    }

    pub(in crate::constraints) fn clamp(
        &self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
    ) -> (f32, f32) {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => {
                self.clamped_clamp(range_min, range_max, value)
            }
            crate::components::RuntimeScrollPhysicsKind::Elastic { .. } => {
                self.elastic_clamp(range_min, range_max, value)
            }
        }
    }

    pub(in crate::constraints) fn run(
        &mut self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
        snapping_points: &[(f32, f32)],
        content_size: f32,
        viewport_size: f32,
    ) {
        if matches!(
            &self.kind,
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. }
        ) {
            self.clamped_run(
                range_min,
                range_max,
                value,
                snapping_points,
                content_size,
                viewport_size,
            );
            return;
        }

        self.elastic_run(
            range_min,
            range_max,
            value,
            snapping_points,
            content_size,
            viewport_size,
        );
    }

    pub(in crate::constraints) fn scroll_physics_run(
        &mut self,
        _range_min: (f32, f32),
        _range_max: (f32, f32),
        _value: (f32, f32),
        _snapping_points: &[(f32, f32)],
        _content_size: f32,
        _viewport_size: f32,
    ) {
        self.is_running = true;
    }

    pub(in crate::constraints) fn advance(&mut self, elapsed_seconds: f32) -> (f32, f32) {
        if matches!(
            &self.kind,
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. }
        ) {
            return self.clamped_advance(elapsed_seconds);
        }

        self.elastic_advance(elapsed_seconds)
    }
}
