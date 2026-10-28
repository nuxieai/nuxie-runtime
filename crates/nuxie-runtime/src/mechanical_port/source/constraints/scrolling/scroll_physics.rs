use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    constraints::draggable_constraint::DraggableConstraintDirection,
    core::{Core, CoreObject, binary_reader::BinaryReader},
    generated::component_base::ComponentBaseCallbacks,
    generated::constraints::scrolling::scroll_physics_base::{
        ScrollPhysicsBase, ScrollPhysicsBaseCallbacks,
    },
    importers::import_stack::ImportStack,
    math::vec2d::Vec2D,
    status_code::StatusCode,
};

impl ComponentBaseCallbacks for ScrollPhysics {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ScrollPhysicsBaseCallbacks for ScrollPhysics {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPhysicsType {
    Elastic,
    Clamped,
    Unknown(u8),
}

impl From<u32> for ScrollPhysicsType {
    fn from(value: u32) -> Self {
        match value as u8 {
            0 => Self::Elastic,
            1 => Self::Clamped,
            value => Self::Unknown(value),
        }
    }
}

/// Handwritten C++ `ScrollPhysics` base retained as an embedded Rust base.
///
/// Concrete virtual behavior is expressed by [`ScrollPhysicsRuntime`].  The
/// state itself remains in the base, exactly as it does in pinned C++, so
/// generated `ClampedScrollPhysicsBase` and `ElasticScrollPhysicsBase` keep
/// the same inheritance/state topology without retained trait-object borrows.
pub struct ScrollPhysics {
    pub base: ScrollPhysicsBase,
    last_time: i64,
    pointer_time: Option<f32>,
    is_running: bool,
    speed: Vec2D,
    acceleration: Vec2D,
    direction: DraggableConstraintDirection,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self {
            base: ScrollPhysicsBase::default(),
            last_time: steady_micros(),
            pointer_time: None,
            is_running: false,
            speed: Vec2D::default(),
            acceleration: Vec2D::default(),
            // Pinned C++ does not read this field until `prepare` writes it.
            // `All` is therefore only the initialized Rust representation of
            // that pre-prepare state, never a behavioral fallback.
            direction: DraggableConstraintDirection::All,
        }
    }
}

impl Deref for ScrollPhysics {
    type Target = ScrollPhysicsBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ScrollPhysics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ScrollPhysics {
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ScrollPhysicsBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScrollPhysicsBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    /// Select the pointer clock for this drag only. A separate floating point
    /// baseline preserves subsecond samples without changing upstream's clock.
    pub fn set_pointer_time(&mut self, time_stamp: Option<f32>) {
        self.pointer_time = time_stamp;
    }

    fn clock_micros(time_stamp: f32) -> i64 {
        if crate::mechanical_port::source::math::random::runtime_deterministic_mode() {
            (time_stamp * 1_000_000.0) as i64
        } else {
            steady_micros()
        }
    }

    /// Prime the clock without measuring a delta from the reset timestamp.
    pub fn mark_time(&mut self, time_stamp: f32) {
        self.last_time = Self::clock_micros(time_stamp);
    }

    pub fn accumulate(&mut self, delta: Vec2D, time_stamp: f32) {
        let elapsed_seconds = if let Some(previous) = self.pointer_time {
            let elapsed = time_stamp - previous;
            if elapsed > 0.0 {
                self.pointer_time = Some(time_stamp);
            }
            elapsed
        } else {
            // Both deterministic stamps and the steady engine clock use microseconds.
            let now = Self::clock_micros(time_stamp);
            let elapsed = now.saturating_sub(self.last_time) as f32 / 1_000_000.0;
            self.last_time = now;
            elapsed
        };
        if elapsed_seconds > 0.0 {
            let last_speed = self.speed;
            self.speed = Vec2D::new(delta.x / elapsed_seconds, delta.y / elapsed_seconds);
            self.acceleration = Vec2D::new(
                (last_speed.x + self.speed.x) / elapsed_seconds,
                (last_speed.y + self.speed.y) / elapsed_seconds,
            );
        }
    }

    pub fn stop_base(&mut self) {
        self.is_running = false;
        self.speed = Vec2D::default();
    }

    pub fn reset_base(&mut self) {
        self.pointer_time = None;
        self.last_time =
            if crate::mechanical_port::source::math::random::runtime_deterministic_mode() {
                0
            } else {
                steady_micros()
            };
        self.speed = Vec2D::default();
        self.acceleration = Vec2D::default();
        self.stop_base();
    }

    pub fn run_base(&mut self) {
        self.is_running = true;
    }

    pub fn speed(&self) -> Vec2D {
        self.speed
    }

    pub fn set_speed(&mut self, speed: Vec2D) {
        self.speed = speed;
    }

    pub fn acceleration(&self) -> Vec2D {
        self.acceleration
    }

    pub fn set_direction(&mut self, direction: DraggableConstraintDirection) {
        self.direction = direction;
    }

    pub fn clear_velocity(&mut self) {
        self.speed = Vec2D::default();
    }

    pub fn claims_artboard_slot(&self, _import_stack: &mut ImportStack) -> bool {
        false
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest_backboard_importer() else {
            return StatusCode::MissingObject;
        };
        let Some(handle) = self.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_physics(handle);
        StatusCode::Ok
    }
}

/// Object-safe counterpart to pinned `ScrollPhysics` virtual dispatch.
pub trait ScrollPhysicsRuntime {
    fn physics(&self) -> &ScrollPhysics;
    fn physics_mut(&mut self) -> &mut ScrollPhysics;

    fn enabled(&self) -> bool {
        self.is_running()
    }

    fn is_running(&self) -> bool {
        self.physics().is_running
    }
    fn is_primed(&self) -> bool {
        true
    }

    fn prepare(&mut self, direction: DraggableConstraintDirection) {
        self.reset();
        self.physics_mut().direction = direction;
    }

    fn clamp(&self, _range_min: Vec2D, _range_max: Vec2D, _value: Vec2D) -> Vec2D {
        Vec2D::default()
    }

    fn advance(&mut self, _elapsed_seconds: f32) -> Vec2D {
        Vec2D::default()
    }
    fn shift(&mut self, _delta: Vec2D) {}

    fn accumulate(&mut self, delta: Vec2D, time_stamp: f32) {
        self.physics_mut().accumulate(delta, time_stamp);
    }

    fn mark_time(&mut self, time_stamp: f32) {
        self.physics_mut().mark_time(time_stamp);
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        _range_min: Vec2D,
        _range_max: Vec2D,
        _value: Vec2D,
        _snapping_points: Vec<Vec2D>,
        _content_size: Vec2D,
        _viewport_size: Vec2D,
    ) {
        self.physics_mut().run_base();
    }

    fn stop(&mut self) {
        self.physics_mut().stop_base();
    }

    fn reset(&mut self) {
        self.physics_mut().reset_base();
    }

    fn speed(&self) -> Vec2D {
        self.physics().speed()
    }

    fn clear_velocity(&mut self) {
        self.physics_mut().clear_velocity();
    }

    #[allow(clippy::too_many_arguments)]
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

pub fn from_core(object: &dyn CoreObject) -> Option<&dyn ScrollPhysicsRuntime> {
    if let Some(physics) = CoreObject::as_any(object)
        .downcast_ref::<crate::mechanical_port::source::constraints::scrolling::clamped_scroll_physics::ClampedScrollPhysics>()
    {
        return Some(physics);
    }
    CoreObject::as_any(object)
        .downcast_ref::<crate::mechanical_port::source::constraints::scrolling::elastic_scroll_physics::ElasticScrollPhysics>()
        .map(|physics| physics as &dyn ScrollPhysicsRuntime)
}

pub fn from_core_mut(object: &mut dyn CoreObject) -> Option<&mut dyn ScrollPhysicsRuntime> {
    if CoreObject::as_any(object).is::<crate::mechanical_port::source::constraints::scrolling::clamped_scroll_physics::ClampedScrollPhysics>() {
        return CoreObject::as_any_mut(object)
            .downcast_mut::<crate::mechanical_port::source::constraints::scrolling::clamped_scroll_physics::ClampedScrollPhysics>()
            .map(|physics| physics as &mut dyn ScrollPhysicsRuntime);
    }
    CoreObject::as_any_mut(object)
        .downcast_mut::<crate::mechanical_port::source::constraints::scrolling::elastic_scroll_physics::ElasticScrollPhysics>()
        .map(|physics| physics as &mut dyn ScrollPhysicsRuntime)
}

fn steady_micros() -> i64 {
    crate::mechanical_port::source::engine_clock::now_micros()
}

#[cfg(test)]
mod clock_contract_tests {
    use super::ScrollPhysics;
    use crate::mechanical_port::source::{engine_clock::with_test_clock, math::vec2d::Vec2D};

    #[test]
    fn pointer_clock_preserves_subseconds_independently_of_arrival_time() {
        fn speeds(clock: fn() -> i64) -> (Vec2D, Vec2D) {
            let mut physics = ScrollPhysics::default();
            physics.set_pointer_time(Some(10.25));
            with_test_clock(clock, || physics.accumulate(Vec2D::new(-60.0, 0.0), 10.30));
            let first = physics.speed();
            with_test_clock(clock, || physics.accumulate(Vec2D::new(-60.0, 0.0), 10.35));
            (first, physics.speed())
        }
        let fast = speeds(|| 124);
        assert!((fast.0.x + 1200.0).abs() < 0.1);
        assert!((fast.1.x + 1200.0).abs() < 0.1);
        assert_eq!(fast, speeds(|| 9_000_000));
    }

    #[test]
    fn scrolling_measures_motion_and_reset_with_the_engine_clock() {
        let mut physics = with_test_clock(|| 1_000_000, ScrollPhysics::default);
        with_test_clock(
            || 1_500_000,
            || physics.accumulate(Vec2D::new(10.0, 20.0), 0.0),
        );
        // Ten and twenty points over half a second, independent of frame time.
        assert_eq!(physics.speed(), Vec2D::new(20.0, 40.0));
        with_test_clock(|| 2_000_000, || physics.reset_base());
        assert_eq!(physics.speed(), Vec2D::default());
        with_test_clock(
            || 2_500_000,
            || physics.accumulate(Vec2D::new(10.0, 20.0), 0.0),
        );
        assert_eq!(physics.speed(), Vec2D::new(20.0, 40.0));
    }
}
