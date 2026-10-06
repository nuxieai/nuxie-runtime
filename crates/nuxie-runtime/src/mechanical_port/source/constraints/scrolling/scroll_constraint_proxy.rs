use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::{DraggableConstraintDirection, DraggableProxy},
        scrolling::scroll_constraint::ScrollConstraint,
    },
    core::CoreHandle,
    drawable::RuntimeDrawableOccurrence,
    math::vec2d::Vec2D,
    scroll_event::{ScrollEvent, ScrollPhase},
};

pub struct ViewportDraggableProxy {
    constraint: CoreHandle,
    hittable: RuntimeDrawableOccurrence,
    last_position: Vec2D,
    is_dragging: bool,
}

impl ViewportDraggableProxy {
    pub fn new(constraint: CoreHandle, hittable: RuntimeDrawableOccurrence) -> Self {
        Self {
            constraint,
            hittable,
            last_position: Vec2D::default(),
            is_dragging: false,
        }
    }
}

impl DraggableProxy for ViewportDraggableProxy {
    fn is_opaque(&self) -> bool {
        false
    }
    fn drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool {
        let handle = self.constraint.clone();
        handle
            .with_downcast_mut::<ScrollConstraint, _>(|constraint| {
                if !constraint.interactive() {
                    return false;
                }
                let delta_position = mouse_position - self.last_position;
                if !self.is_dragging {
                    let crossed = match constraint.direction() {
                        DraggableConstraintDirection::Vertical => {
                            Some(delta_position.y.abs() > constraint.threshold())
                        }
                        DraggableConstraintDirection::Horizontal => {
                            Some(delta_position.x.abs() > constraint.threshold())
                        }
                        DraggableConstraintDirection::All => {
                            Some(delta_position.length() > constraint.threshold())
                        }
                        _ => None,
                    };
                    if let Some(crossed) = crossed {
                        if crossed {
                            self.is_dragging = true;
                        } else {
                            return false;
                        }
                    }
                }
                constraint.drag_view(delta_position, time_stamp, true);
                self.last_position = mouse_position;
                true
            })
            .expect("live ScrollConstraint occurrence")
    }
    fn start_drag(&mut self, mouse_position: Vec2D, _time_stamp: f32) -> bool {
        let handle = self.constraint.clone();
        handle
            .with_downcast_mut::<ScrollConstraint, _>(|constraint| {
                if !constraint.interactive() {
                    return false;
                }
                self.is_dragging = false;
                constraint.init_physics();
                self.last_position = mouse_position;
                true
            })
            .expect("live ScrollConstraint occurrence")
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        self.constraint
            .with_downcast_mut::<ScrollConstraint, _>(|constraint| {
                if !constraint.interactive() {
                    return false;
                }
                constraint.run_physics();
                true
            })
            .expect("live ScrollConstraint occurrence")
    }
    fn hittable(&self) -> Option<RuntimeDrawableOccurrence> {
        Some(self.hittable.clone())
    }
    fn wants_scroll(&mut self, event: &ScrollEvent) -> bool {
        self.constraint
            .with_downcast::<ScrollConstraint, _>(|c| {
                if event.phase == ScrollPhase::InertiaCancel {
                    return c.wheel_enabled() && (c.is_scrolling() || physics_running(c));
                }
                if event.precise {
                    return c.can_consume(event.delta) || c.can_stretch(event.delta);
                }
                c.can_consume(event.delta)
            })
            .expect("live ScrollConstraint occurrence")
    }
    fn accepts_scroll(&mut self) -> bool {
        self.constraint
            .with_downcast::<ScrollConstraint, _>(ScrollConstraint::wheel_enabled)
            .expect("live ScrollConstraint occurrence")
    }
    fn cancel_scroll(&mut self) {
        self.constraint
            .with_downcast_mut::<ScrollConstraint, _>(|c| {
                c.stop_physics();
                c.clear_velocity();
                c.end_scroll_gesture();
            })
            .expect("live ScrollConstraint occurrence");
    }
    fn is_scroll_gesture_active(&mut self) -> bool {
        self.constraint
            .with_downcast::<ScrollConstraint, _>(ScrollConstraint::is_scrolling)
            .expect("live ScrollConstraint occurrence")
    }
    fn scroll(&mut self, event: &ScrollEvent, time_stamp: f32) -> bool {
        self.constraint
            .with_downcast_mut::<ScrollConstraint, _>(|c| {
                if !c.wheel_enabled() {
                    return false;
                }
                match event.phase {
                    ScrollPhase::InertiaCancel => {
                        c.stop_physics();
                        c.clear_velocity();
                        c.end_scroll_gesture();
                    }
                    ScrollPhase::Begin => {
                        c.begin_scroll_gesture();
                        c.prime_physics();
                    }
                    ScrollPhase::Update => {
                        if event.precise {
                            let started = c.begin_scroll_gesture();
                            let running = physics_running(c);
                            let mut primed = started || running;
                            if primed {
                                c.prime_physics();
                            } else {
                                primed = c.ensure_physics_primed();
                            }
                            c.drag_view(event.delta, time_stamp, !primed);
                        } else {
                            if physics_running(c) {
                                c.stop_physics();
                            }
                            c.begin_scroll_gesture();
                            c.scroll_by(event.delta);
                        }
                    }
                    ScrollPhase::Momentum => {
                        let settling = physics_running(c);
                        if c.is_overscrolled() || settling {
                            if !settling {
                                c.prime_physics();
                                c.start_physics();
                            }
                            c.mark_scroll_activity();
                            return true;
                        }
                        c.begin_scroll_gesture();
                        c.scroll_by(event.delta);
                    }
                    ScrollPhase::End => {
                        c.start_physics();
                        c.end_scroll_gesture();
                    }
                }
                true
            })
            .expect("live ScrollConstraint occurrence")
    }
}

fn physics_running(constraint: &ScrollConstraint) -> bool {
    constraint.physics().is_some_and(|physics| {
        physics
            .with(|object| {
                super::scroll_physics::from_core(object)
                    .expect("ScrollPhysics-derived occurrence")
                    .is_running()
            })
            .expect("live ScrollPhysics occurrence")
    })
}
