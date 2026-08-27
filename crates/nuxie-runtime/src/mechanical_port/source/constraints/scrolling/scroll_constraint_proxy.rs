use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::{DraggableConstraintDirection, DraggableProxy},
        scrolling::scroll_constraint::ScrollConstraint,
    },
    drawable::Drawable,
    math::vec2d::Vec2D,
};

pub struct ViewportDraggableProxy {
    constraint: *mut ScrollConstraint,
    hittable: *mut Drawable,
    last_position: Vec2D,
    is_dragging: bool,
}

impl ViewportDraggableProxy {
    pub fn new(constraint: &mut ScrollConstraint, hittable: &mut Drawable) -> Self {
        Self { constraint, hittable, last_position: Vec2D::default(), is_dragging: false }
    }
}

impl DraggableProxy for ViewportDraggableProxy {
    fn is_opaque(&self) -> bool { false }
    fn drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool {
        let constraint = unsafe { &mut *self.constraint };
        if !constraint.interactive() { return false; }
        let delta_position = mouse_position - self.last_position;
        if !self.is_dragging {
            let crossed = match constraint.direction() {
                DraggableConstraintDirection::Vertical => delta_position.y.abs() > constraint.threshold(),
                DraggableConstraintDirection::Horizontal => delta_position.x.abs() > constraint.threshold(),
                DraggableConstraintDirection::All => delta_position.length() > constraint.threshold(),
            };
            if crossed { self.is_dragging = true; } else { return false; }
        }
        constraint.drag_view(delta_position, time_stamp);
        self.last_position = mouse_position;
        true
    }
    fn start_drag(&mut self, mouse_position: Vec2D, _time_stamp: f32) -> bool {
        let constraint = unsafe { &mut *self.constraint };
        if !constraint.interactive() { return false; }
        self.is_dragging = false;
        constraint.init_physics();
        self.last_position = mouse_position;
        true
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        let constraint = unsafe { &mut *self.constraint };
        if !constraint.interactive() { return false; }
        constraint.run_physics();
        true
    }
    fn hittable(&mut self) -> Option<&mut Drawable> { Some(unsafe { &mut *self.hittable }) }
}
