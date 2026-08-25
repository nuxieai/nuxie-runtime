//! Direct owner for pinned `src/constraints/scrolling/elastic_scroll_physics.cpp`.

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
            self.speed += -self.speed * (elapsed_seconds * self.friction).min(1.0);
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
            self.current += diff * (elapsed_seconds * 15.0).min(1.0);
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
        self.target = value.clamp(range_min, range_max);
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
            (end_target / section_size).floor()
        } else {
            0.0
        };
        let mod_end_target = if range_max == f32::INFINITY {
            ((end_target % section_size) + section_size) % section_size
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
                snap_target = *snap + multiple * section_size;
            }
        }
        if max_target != f32::INFINITY {
            let diff = (max_target - mod_end_target).abs();
            if diff < closest {
                snap_target = max_target;
            }
        }
        snap_target = snap_target.min(max_target);
        self.speed = -(snap_target + self.current) * self.friction;
        self.snap_target = -snap_target;
    }
}
