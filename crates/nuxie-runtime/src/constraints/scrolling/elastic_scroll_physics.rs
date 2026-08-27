//! Direct owner for pinned `src/constraints/scrolling/elastic_scroll_physics.cpp`.

use super::super::*;

fn positive_mod(mut value: f32, mut range: f32) -> f32 {
    if range < 0.0 {
        range = -range;
    }
    value %= range;
    if value < 0.0 {
        value += range;
    }
    value
}

fn cpp_std_min(lhs: f32, rhs: f32) -> f32 {
    if rhs < lhs { rhs } else { lhs }
}

impl crate::components::RuntimeElasticScrollPhysicsHelper {
    pub(in crate::constraints) fn scroll_to(
        &mut self,
        current: f32,
        target: f32,
        range_min: f32,
        range_max: f32,
    ) {
        self.is_running = true;
        self.run_range_min = range_min;
        self.run_range_max = range_max;
        self.current = current;
        self.target = target;
        self.speed = 0.0;
        self.snap_target = f32::NAN;
    }

    pub(in crate::constraints) fn advance(&mut self, elapsed_seconds: f32) -> f32 {
        if self.speed != 0.0 {
            self.current += self.speed * elapsed_seconds;
            if self.current < self.run_range_min {
                self.friction *= 4.0;
            } else if self.current > self.run_range_max {
                self.friction *= 4.0;
            }
            self.speed += -self.speed * cpp_std_min(1.0, elapsed_seconds * self.friction);
            if self.speed.abs() < 5.0 {
                self.speed = 0.0;
                self.target = if self.current < self.run_range_min {
                    self.run_range_min
                } else if self.current > self.run_range_max {
                    self.run_range_max
                } else {
                    self.current
                };
            }
            return self.current;
        }
        let diff = self.target - self.current;
        if diff.abs() < 0.1 {
            self.current = if self.snap_target.is_nan() {
                self.target
            } else {
                self.snap_target
            };
            self.is_running = false;
        } else {
            self.current += diff * cpp_std_min(1.0, elapsed_seconds * 15.0);
        }
        self.current
    }

    pub(in crate::constraints) fn clamp(&self, range_min: f32, range_max: f32, value: f32) -> f32 {
        if value < range_min {
            range_min - (-(value - range_min)).powf(self.elastic_factor)
        } else if value > range_max {
            // Preserve pinned C++'s literal `value + rangeMax`, including its
            // asymmetric behavior for non-zero maxima.
            range_max + (value + range_max).powf(self.elastic_factor)
        } else {
            value
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::constraints) fn run(
        &mut self,
        acceleration: f32,
        range_min: f32,
        range_max: f32,
        value: f32,
        snapping_points: &[f32],
        content_size: f32,
        viewport_size: f32,
    ) {
        let _ = viewport_size;
        self.is_running = true;
        self.run_range_min = range_min;
        self.run_range_max = range_max;
        self.speed = if acceleration.abs() > 100.0 {
            acceleration * 0.16 * 0.16 * 0.1 * self.speed_multiplier
        } else {
            0.0
        };
        // Pinned C++ uses comparisons rather than `std::clamp`. Besides
        // preserving NaN, this remains defined when malformed bounds arrive
        // reversed; Rust's `f32::clamp` would panic in that case.
        self.target = if value < range_min {
            range_min
        } else if value > range_max {
            range_max
        } else {
            value
        };
        self.current = value;
        if snapping_points.is_empty() {
            self.snap_target = f32::NAN;
            return;
        }
        let end_target = -(self.current + self.speed / self.friction);
        let section_size = if content_size != 0.0 {
            content_size
        } else {
            1.0
        };
        let multiple = if range_max == f32::INFINITY {
            (end_target / section_size).floor() as i32
        } else {
            0
        };
        let mod_end_target = if range_max == f32::INFINITY {
            positive_mod(end_target, section_size)
        } else {
            end_target
        };
        let max_target = if range_max == f32::INFINITY {
            f32::INFINITY
        } else {
            -range_min
        };
        let mut closest = f32::MAX;
        let mut snap_target = 0.0;
        for snap in snapping_points {
            let diff = (*snap - mod_end_target).abs();
            if diff < closest {
                closest = diff;
                snap_target = *snap + multiple as f32 * section_size;
            }
        }
        if max_target != f32::INFINITY {
            let diff = (max_target - mod_end_target).abs();
            if diff < closest {
                snap_target = max_target;
            }
        }
        snap_target = cpp_std_min(snap_target, max_target);
        self.speed = -(snap_target + self.current) * self.friction;
        self.snap_target = -snap_target;
    }
}

impl RuntimeScrollPhysicsState {
    pub(in crate::constraints) fn elastic_enabled(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.is_some() || y.is_some()
            }
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_is_running(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.as_ref().is_some_and(|helper| helper.is_running)
                    || y.as_ref().is_some_and(|helper| helper.is_running)
            }
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_target_value(&self, axis: RuntimeScrollAxis) -> f32 {
        match (&self.kind, axis) {
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { x, .. },
                RuntimeScrollAxis::X,
            ) => x.as_ref().map_or(0.0, |helper| helper.target),
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { y, .. },
                RuntimeScrollAxis::Y,
            ) => y.as_ref().map_or(0.0, |helper| helper.target),
            (crate::components::RuntimeScrollPhysicsKind::Clamped { .. }, _) => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_has_target(&self, axis: RuntimeScrollAxis) -> bool {
        match (&self.kind, axis) {
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { x, .. },
                RuntimeScrollAxis::X,
            ) => x.is_some(),
            (
                crate::components::RuntimeScrollPhysicsKind::Elastic { y, .. },
                RuntimeScrollAxis::Y,
            ) => y.is_some(),
            (crate::components::RuntimeScrollPhysicsKind::Clamped { .. }, _) => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_reset(&mut self) {
        self.scroll_physics_reset();
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                *x = None;
                *y = None;
            }
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_prepare(&mut self, direction: u64) {
        self.scroll_physics_prepare(direction);
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
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
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_scroll_to_position(
        &mut self,
        current: (f32, f32),
        target: (f32, f32),
        range_min: (f32, f32),
        range_max: (f32, f32),
        horizontal: bool,
        vertical: bool,
    ) {
        let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind else {
            unreachable!();
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

    pub(in crate::constraints) fn elastic_clamp(
        &self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
    ) -> (f32, f32) {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => (
                x.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.0, range_max.0, value.0)
                }),
                y.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.1, range_max.1, value.1)
                }),
            ),
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_run(
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
        let mut x_points = Vec::with_capacity(snapping_points.len());
        let mut y_points = Vec::with_capacity(snapping_points.len());
        for point in snapping_points {
            x_points.push(point.0);
            y_points.push(point.1);
        }
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
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
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        }
    }

    pub(in crate::constraints) fn elastic_advance(&mut self, elapsed_seconds: f32) -> (f32, f32) {
        let (result, running) = match &mut self.kind {
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
                (result, running)
            }
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => unreachable!(),
        };
        if !running {
            self.elastic_reset();
        }
        result
    }
}
