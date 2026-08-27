use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::DraggableProxy, scrolling::scroll_bar_constraint::ScrollBarConstraint,
    },
    drawable::Drawable,
    math::vec2d::Vec2D,
};

pub struct ThumbDraggableProxy {
    constraint: *mut ScrollBarConstraint,
    hittable: *mut Drawable,
    last_position: Vec2D,
}

impl ThumbDraggableProxy {
    pub fn new(constraint: &mut ScrollBarConstraint, hittable: &mut Drawable) -> Self {
        Self {
            constraint,
            hittable,
            last_position: Vec2D::default(),
        }
    }
}

impl DraggableProxy for ThumbDraggableProxy {
    fn is_opaque(&self) -> bool {
        true
    }
    fn drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool {
        unsafe { (*self.constraint).drag_thumb(mouse_position - self.last_position, time_stamp) };
        self.last_position = mouse_position;
        true
    }
    fn start_drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool {
        self.last_position = mouse_position;
        let scroll = unsafe { (*self.constraint).scroll_constraint_mut() };
        scroll.set_is_scroll_bar_dragging(true);
        if let Some(physics) = scroll.physics_mut() {
            physics.accumulate(Vec2D::default(), time_stamp);
        }
        true
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        let scroll = unsafe { (*self.constraint).scroll_constraint_mut() };
        scroll.set_is_scroll_bar_dragging(false);
        scroll.clear_velocity();
        true
    }
    fn hittable(&mut self) -> Option<&mut Drawable> {
        Some(unsafe { &mut *self.hittable })
    }
}

pub struct TrackDraggableProxy {
    constraint: *mut ScrollBarConstraint,
    hittable: *mut Drawable,
}
impl TrackDraggableProxy {
    pub fn new(constraint: &mut ScrollBarConstraint, hittable: &mut Drawable) -> Self {
        Self {
            constraint,
            hittable,
        }
    }
}
impl DraggableProxy for TrackDraggableProxy {
    fn start_drag(&mut self, mouse_position: Vec2D, _time_stamp: f32) -> bool {
        unsafe { (*self.constraint).hit_track(mouse_position) };
        true
    }
    fn drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        true
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        true
    }
    fn hittable(&mut self) -> Option<&mut Drawable> {
        Some(unsafe { &mut *self.hittable })
    }
}
