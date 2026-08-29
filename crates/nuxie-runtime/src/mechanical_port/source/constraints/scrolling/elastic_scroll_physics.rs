use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::DraggableConstraintDirection,
        scrolling::scroll_physics::{ScrollPhysics, ScrollPhysicsRuntime},
    },
    generated::constraints::scrolling::elastic_scroll_physics_base::ElasticScrollPhysicsBase,
    math::vec2d::Vec2D,
};

pub struct ElasticScrollPhysicsHelper {
    friction: f32,
    speed_multiplier: f32,
    elastic_factor: f32,
    target: f32,
    current: f32,
    speed: f32,
    snap_target: f32,
    run_range_min: f32,
    run_range_max: f32,
    is_running: bool,
}

impl ElasticScrollPhysicsHelper {
    pub fn new(friction: f32, speed_multiplier: f32, elastic_factor: f32) -> Self {
        Self {
            friction,
            speed_multiplier,
            elastic_factor,
            target: 0.0,
            current: 0.0,
            speed: 0.0,
            snap_target: f32::NAN,
            run_range_min: 0.0,
            run_range_max: 0.0,
            is_running: false,
        }
    }
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    pub fn target(&self) -> f32 {
        self.target
    }
    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> f32 {
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
                if self.current < self.run_range_min {
                    self.target = self.run_range_min;
                } else if self.current > self.run_range_max {
                    self.target = self.run_range_max;
                } else {
                    self.target = self.current;
                }
            }
            return self.current;
        }
        let difference = self.target - self.current;
        if difference.abs() < 0.1 {
            self.current = if self.snap_target.is_nan() {
                self.target
            } else {
                self.snap_target
            };
            self.is_running = false;
        } else {
            self.current += difference * (elapsed_seconds * 15.0).min(1.0);
        }
        self.current
    }

    pub fn clamp(&self, range_min: f32, range_max: f32, value: f32) -> f32 {
        if value < range_min {
            range_min - (-(value - range_min)).powf(self.elastic_factor)
        } else if value > range_max {
            // Preserve the pinned `value + rangeMax` expression verbatim.
            range_max + (value + range_max).powf(self.elastic_factor)
        } else {
            value
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        acceleration: f32,
        range_min: f32,
        range_max: f32,
        value: f32,
        snapping_points: Vec<f32>,
        content_size: f32,
        _viewport_size: f32,
    ) {
        self.is_running = true;
        self.run_range_min = range_min;
        self.run_range_max = range_max;
        self.speed = if acceleration.abs() > 100.0 {
            acceleration * 0.16 * 0.16 * 0.1 * self.speed_multiplier
        } else {
            0.0
        };
        self.target = if value < range_min {
            range_min
        } else if value > range_max {
            range_max
        } else {
            value
        };
        self.current = value;
        if !snapping_points.is_empty() {
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
                end_target.rem_euclid(section_size)
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
                let difference = (snap - mod_end_target).abs();
                if difference < closest {
                    closest = difference;
                    snap_target = snap + multiple as f32 * section_size;
                }
            }
            if max_target != f32::INFINITY {
                let difference = (max_target - mod_end_target).abs();
                if difference < closest {
                    snap_target = max_target;
                }
            }
            snap_target = snap_target.min(max_target);
            self.speed = -(snap_target + self.current) * self.friction;
            self.snap_target = -snap_target;
        } else {
            self.snap_target = f32::NAN;
        }
    }

    pub fn scroll_to(&mut self, current: f32, target: f32, range_min: f32, range_max: f32) {
        self.is_running = true;
        self.run_range_min = range_min;
        self.run_range_max = range_max;
        self.current = current;
        self.target = target;
        self.speed = 0.0;
        self.snap_target = f32::NAN;
    }
}

pub struct ElasticScrollPhysics {
    pub base: ElasticScrollPhysicsBase,
    physics_x: Option<Box<ElasticScrollPhysicsHelper>>,
    physics_y: Option<Box<ElasticScrollPhysicsHelper>>,
}

impl Default for ElasticScrollPhysics {
    fn default() -> Self {
        Self {
            base: ElasticScrollPhysicsBase::default(),
            physics_x: None,
            physics_y: None,
        }
    }
}

impl ElasticScrollPhysics {
    pub fn enabled(&self) -> bool {
        self.physics_x.is_some() || self.physics_y.is_some()
    }
    pub fn is_running(&self) -> bool {
        self.physics_x.as_ref().is_some_and(|p| p.is_running())
            || self.physics_y.as_ref().is_some_and(|p| p.is_running())
    }
    pub fn target_x(&self) -> f32 {
        self.physics_x.as_ref().map_or(0.0, |p| p.target())
    }
    pub fn target_y(&self) -> f32 {
        self.physics_y.as_ref().map_or(0.0, |p| p.target())
    }
    pub fn has_target_x(&self) -> bool {
        self.physics_x.is_some()
    }
    pub fn has_target_y(&self) -> bool {
        self.physics_y.is_some()
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> Vec2D {
        let previous_x = self.physics_x.as_ref().map_or(0.0, |p| p.current());
        let previous_y = self.physics_y.as_ref().map_or(0.0, |p| p.current());
        let advance_x = self
            .physics_x
            .as_mut()
            .map_or(0.0, |p| p.advance(elapsed_seconds));
        let advance_y = self
            .physics_y
            .as_mut()
            .map_or(0.0, |p| p.advance(elapsed_seconds));
        if elapsed_seconds > 0.0 {
            self.base.base.set_speed(Vec2D::new(
                (advance_x - previous_x) / elapsed_seconds,
                (advance_y - previous_y) / elapsed_seconds,
            ));
        }
        let running_x = self.physics_x.as_ref().is_some_and(|p| p.is_running());
        let running_y = self.physics_y.as_ref().is_some_and(|p| p.is_running());
        if !running_x && !running_y {
            self.reset();
        }
        Vec2D::new(advance_x, advance_y)
    }

    pub fn clamp(&self, range_min: Vec2D, range_max: Vec2D, value: Vec2D) -> Vec2D {
        Vec2D::new(
            self.physics_x
                .as_ref()
                .map_or(0.0, |p| p.clamp(range_min.x, range_max.x, value.x)),
            self.physics_y
                .as_ref()
                .map_or(0.0, |p| p.clamp(range_min.y, range_max.y, value.y)),
        )
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
        let x_points = snapping_points.iter().map(|point| point.x).collect();
        let y_points = snapping_points.iter().map(|point| point.y).collect();
        if let Some(physics) = &mut self.physics_x {
            physics.run(
                self.base.base.acceleration().x,
                range_min.x,
                range_max.x,
                value.x,
                x_points,
                content_size,
                viewport_size,
            );
        }
        if let Some(physics) = &mut self.physics_y {
            physics.run(
                self.base.base.acceleration().y,
                range_min.y,
                range_max.y,
                value.y,
                y_points,
                content_size,
                viewport_size,
            );
        }
    }

    pub fn prepare(&mut self, direction: DraggableConstraintDirection) {
        // `ScrollPhysics::prepare` invokes virtual `reset` before assigning the
        // direction. Preserve that ordering explicitly in Rust.
        self.reset();
        self.base.base.set_direction(direction);
        if matches!(
            direction,
            DraggableConstraintDirection::Horizontal | DraggableConstraintDirection::All
        ) {
            self.physics_x = Some(Box::new(ElasticScrollPhysicsHelper::new(
                self.base.friction(),
                self.base.speed_multiplier(),
                self.base.elastic_factor(),
            )));
        }
        if matches!(
            direction,
            DraggableConstraintDirection::Vertical | DraggableConstraintDirection::All
        ) {
            self.physics_y = Some(Box::new(ElasticScrollPhysicsHelper::new(
                self.base.friction(),
                self.base.speed_multiplier(),
                self.base.elastic_factor(),
            )));
        }
    }

    pub fn reset(&mut self) {
        self.base.base.reset_base();
        self.physics_x = None;
        self.physics_y = None;
    }

    pub fn scroll_to_position(
        &mut self,
        current: Vec2D,
        target: Vec2D,
        range_min: Vec2D,
        range_max: Vec2D,
        horizontal: bool,
        vertical: bool,
    ) {
        if horizontal && self.physics_x.is_none() {
            self.physics_x = Some(Box::new(ElasticScrollPhysicsHelper::new(
                self.base.friction(),
                self.base.speed_multiplier(),
                self.base.elastic_factor(),
            )));
        }
        if vertical && self.physics_y.is_none() {
            self.physics_y = Some(Box::new(ElasticScrollPhysicsHelper::new(
                self.base.friction(),
                self.base.speed_multiplier(),
                self.base.elastic_factor(),
            )));
        }
        if horizontal {
            if let Some(physics) = &mut self.physics_x {
                physics.scroll_to(current.x, target.x, range_min.x, range_max.x);
            }
        }
        if vertical {
            if let Some(physics) = &mut self.physics_y {
                physics.scroll_to(current.y, target.y, range_min.y, range_max.y);
            }
        }
    }
}

impl ScrollPhysicsRuntime for ElasticScrollPhysics {
    fn physics(&self) -> &ScrollPhysics {
        &self.base.base
    }

    fn physics_mut(&mut self) -> &mut ScrollPhysics {
        &mut self.base.base
    }

    fn enabled(&self) -> bool {
        ElasticScrollPhysics::enabled(self)
    }

    fn is_running(&self) -> bool {
        ElasticScrollPhysics::is_running(self)
    }

    fn target_x(&self) -> f32 {
        ElasticScrollPhysics::target_x(self)
    }

    fn target_y(&self) -> f32 {
        ElasticScrollPhysics::target_y(self)
    }

    fn has_target_x(&self) -> bool {
        ElasticScrollPhysics::has_target_x(self)
    }

    fn has_target_y(&self) -> bool {
        ElasticScrollPhysics::has_target_y(self)
    }

    fn advance(&mut self, elapsed_seconds: f32) -> Vec2D {
        ElasticScrollPhysics::advance(self, elapsed_seconds)
    }

    fn clamp(&self, range_min: Vec2D, range_max: Vec2D, value: Vec2D) -> Vec2D {
        ElasticScrollPhysics::clamp(self, range_min, range_max, value)
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
        ElasticScrollPhysics::run(
            self,
            range_min,
            range_max,
            value,
            snapping_points,
            content_size,
            viewport_size,
        );
    }

    fn prepare(&mut self, direction: DraggableConstraintDirection) {
        ElasticScrollPhysics::prepare(self, direction);
    }

    fn reset(&mut self) {
        ElasticScrollPhysics::reset(self);
    }

    fn scroll_to_position(
        &mut self,
        current: Vec2D,
        target: Vec2D,
        range_min: Vec2D,
        range_max: Vec2D,
        horizontal: bool,
        vertical: bool,
    ) {
        ElasticScrollPhysics::scroll_to_position(
            self, current, target, range_min, range_max, horizontal, vertical,
        );
    }
}
