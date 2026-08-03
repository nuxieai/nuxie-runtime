//! Shared direct owner for pinned `src/constraints/scrolling/scroll_physics.cpp`
//! and its inseparable `clamped_scroll_physics.cpp` / `elastic_scroll_physics.cpp`
//! variant branches.

use super::super::*;

impl RuntimeScrollPhysicsState {
    pub(in crate::constraints) fn enabled(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.is_some() || y.is_some()
            }
        }
    }

    pub(in crate::constraints) fn is_running(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.as_ref().is_some_and(|helper| helper.is_running)
                    || y.as_ref().is_some_and(|helper| helper.is_running)
            }
        }
    }

    pub(in crate::constraints) fn stop(&mut self) {
        self.is_running = false;
        self.speed = (0.0, 0.0);
    }

    pub(in crate::constraints) fn reset(&mut self) {
        self.last_time_micros = 0;
        self.speed = (0.0, 0.0);
        self.acceleration = (0.0, 0.0);
        self.stop();
        if let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind {
            *x = None;
            *y = None;
        }
    }

    pub(in crate::constraints) fn prepare(&mut self, direction: u64) {
        self.reset();
        self.direction = direction;
        if let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind {
            if matches!(direction, 0 | 2) {
                *x = Some(crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                ));
            }
            if matches!(direction, 1 | 2) {
                *y = Some(crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                ));
            }
        }
    }

    pub(in crate::constraints) fn clear_velocity(&mut self) {
        self.speed = (0.0, 0.0);
    }

    pub(in crate::constraints) fn target(&self, axis: RuntimeScrollAxis) -> Option<f32> {
        if !self.is_running() {
            return None;
        }
        match (&self.kind, axis) {
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { x, .. },
                RuntimeScrollAxis::X,
            ) => x.as_ref().map(|helper| helper.target),
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { y, .. },
                RuntimeScrollAxis::Y,
            ) => y.as_ref().map(|helper| helper.target),
            (crate::components::RuntimeScrollPhysicsKind::Clamped { .. }, _) => None,
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
        let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind else {
            // Pinned base ScrollPhysics and ClampedScrollPhysics do not
            // override scrollToPosition.
            return;
        };
        if horizontal {
            let helper = x.get_or_insert_with(|| {
                crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                )
            });
            helper.scroll_to(current.0, target.0, range_min.0, range_max.0);
        }
        if vertical {
            let helper = y.get_or_insert_with(|| {
                crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                )
            });
            helper.scroll_to(current.1, target.1, range_min.1, range_max.1);
        }
    }

    pub(in crate::constraints) fn accumulate(&mut self, delta: (f32, f32), timestamp: f32) {
        // Canonical runtime/probe execution uses C++ deterministicMode: the
        // pointer timestamp is the clock and reset seeds zero
        // (`scroll_physics.cpp:8-34,36-51`).
        let elapsed_seconds = timestamp - self.last_time_micros as f32;
        self.last_time_micros = timestamp as i64;
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
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => (
                value.0.clamp(range_min.0, range_max.0),
                value.1.clamp(range_min.1, range_max.1),
            ),
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => (
                x.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.0, range_max.0, value.0)
                }),
                y.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.1, range_max.1, value.1)
                }),
            ),
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
        self.is_running = true;
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value: retained } => {
                *retained = (
                    value.0.clamp(range_min.0, range_max.0),
                    value.1.clamp(range_min.1, range_max.1),
                );
            }
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                let x_points = snapping_points
                    .iter()
                    .map(|point| point.0)
                    .collect::<Vec<_>>();
                let y_points = snapping_points
                    .iter()
                    .map(|point| point.1)
                    .collect::<Vec<_>>();
                if let Some(helper) = x {
                    helper.run(
                        self.acceleration.0,
                        range_min.0,
                        range_max.0,
                        value.0,
                        &x_points,
                        content_size,
                        viewport_size,
                    );
                }
                if let Some(helper) = y {
                    helper.run(
                        self.acceleration.1,
                        range_min.1,
                        range_max.1,
                        value.1,
                        &y_points,
                        content_size,
                        viewport_size,
                    );
                }
            }
        }
    }

    pub(in crate::constraints) fn advance(&mut self, elapsed_seconds: f32) -> (f32, f32) {
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value } => {
                let result = *value;
                self.stop();
                result
            }
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                let previous = (
                    x.as_ref().map_or(0.0, |helper| helper.current),
                    y.as_ref().map_or(0.0, |helper| helper.current),
                );
                let result = (
                    x.as_mut()
                        .map_or(0.0, |helper| helper.advance(elapsed_seconds)),
                    y.as_mut()
                        .map_or(0.0, |helper| helper.advance(elapsed_seconds)),
                );
                if elapsed_seconds > 0.0 {
                    self.speed = (
                        (result.0 - previous.0) / elapsed_seconds,
                        (result.1 - previous.1) / elapsed_seconds,
                    );
                }
                let running = x.as_ref().is_some_and(|helper| helper.is_running)
                    || y.as_ref().is_some_and(|helper| helper.is_running);
                if !running {
                    self.reset();
                }
                result
            }
        }
    }
}
