use std::time::{SystemTime, UNIX_EPOCH};

use crate::mechanical_port::source::{
    backboard::BackboardBase,
    constraints::draggable_constraint::DraggableConstraintDirection,
    core_context::StatusCode,
    file::File,
    generated::constraints::scrolling::scroll_physics_base::ScrollPhysicsBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    math::vec2d::Vec2D,
};

fn now_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}

pub struct ScrollPhysicsState {
    last_time: i64,
    is_running: bool,
    pub speed: Vec2D,
    pub acceleration: Vec2D,
    pub direction: DraggableConstraintDirection,
}

impl Default for ScrollPhysicsState {
    fn default() -> Self {
        Self {
            last_time: now_micros(),
            is_running: false,
            speed: Vec2D::default(),
            acceleration: Vec2D::default(),
            direction: DraggableConstraintDirection::Horizontal,
        }
    }
}

pub trait ScrollPhysics: ScrollPhysicsBase {
    fn physics_state(&self) -> &ScrollPhysicsState;
    fn physics_state_mut(&mut self) -> &mut ScrollPhysicsState;
    fn enabled(&self) -> bool {
        self.is_running()
    }
    fn is_running(&self) -> bool {
        self.physics_state().is_running
    }
    fn prepare(&mut self, direction: DraggableConstraintDirection) {
        self.reset();
        self.physics_state_mut().direction = direction;
    }
    fn clamp(&self, _range_min: Vec2D, _range_max: Vec2D, _value: Vec2D) -> Vec2D {
        Vec2D::default()
    }
    fn advance(&mut self, _elapsed_seconds: f32) -> Vec2D {
        Vec2D::default()
    }

    fn accumulate(&mut self, delta: Vec2D, time_stamp: f32) {
        let elapsed_seconds;
        if File::deterministic_mode() {
            elapsed_seconds = time_stamp - self.physics_state().last_time as f32;
            self.physics_state_mut().last_time = time_stamp as i64;
        } else {
            let now = now_micros();
            elapsed_seconds = (now - self.physics_state().last_time) as f32 / 1_000_000.0;
            self.physics_state_mut().last_time = now;
        }
        if elapsed_seconds > 0.0 {
            let last_speed = self.physics_state().speed;
            let speed = Vec2D::new(delta.x / elapsed_seconds, delta.y / elapsed_seconds);
            self.physics_state_mut().speed = speed;
            self.physics_state_mut().acceleration = Vec2D::new(
                (last_speed.x + speed.x) / elapsed_seconds,
                (last_speed.y + speed.y) / elapsed_seconds,
            );
        }
    }

    fn run(
        &mut self,
        _range_min: Vec2D,
        _range_max: Vec2D,
        _value: Vec2D,
        _snapping_points: Vec<Vec2D>,
        _content_size: f32,
        _viewport_size: f32,
    ) {
        self.physics_state_mut().is_running = true;
    }
    fn stop(&mut self) {
        self.physics_state_mut().is_running = false;
        self.physics_state_mut().speed = Vec2D::default();
    }
    fn reset(&mut self) {
        self.physics_state_mut().last_time = if File::deterministic_mode() {
            0
        } else {
            now_micros()
        };
        self.physics_state_mut().speed = Vec2D::default();
        self.physics_state_mut().acceleration = Vec2D::default();
        self.stop();
    }
    fn speed(&self) -> Vec2D {
        self.physics_state().speed
    }
    fn clear_velocity(&mut self) {
        self.physics_state_mut().speed = Vec2D::default();
    }

    fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if let Some(importer) =
            import_stack.latest_mut::<BackboardImporter>(BackboardBase::TYPE_KEY)
        {
            importer.add_physics(self.as_scroll_physics_mut_ptr());
        } else {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }

    fn scroll_to_position(
        &mut self,
        _current: Vec2D,
        _target: Vec2D,
        _range_min: Vec2D,
        _range_max: Vec2D,
        _horizontal: bool,
        _vertical: bool,
    ) {
    }
    fn target_x(&self) -> f32 {
        0.0
    }
    fn target_y(&self) -> f32 {
        0.0
    }
    fn has_target_x(&self) -> bool {
        false
    }
    fn has_target_y(&self) -> bool {
        false
    }
}
