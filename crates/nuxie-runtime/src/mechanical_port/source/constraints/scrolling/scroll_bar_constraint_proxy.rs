use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::DraggableProxy, scrolling::scroll_bar_constraint::ScrollBarConstraint,
    },
    core::CoreHandle,
    math::vec2d::Vec2D,
};

pub struct ThumbDraggableProxy {
    constraint: CoreHandle,
    hittable: CoreHandle,
    last_position: Vec2D,
}

impl ThumbDraggableProxy {
    pub fn new(constraint: CoreHandle, hittable: CoreHandle) -> Self {
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
        let delta = mouse_position - self.last_position;
        self.constraint
            .with_downcast_mut::<ScrollBarConstraint, _>(|constraint| {
                constraint.drag_thumb(delta, time_stamp)
            })
            .expect("live ScrollBarConstraint occurrence");
        self.last_position = mouse_position;
        true
    }
    fn start_drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool {
        self.last_position = mouse_position;
        self.constraint
            .with_downcast_mut::<ScrollBarConstraint, _>(|constraint| {
                constraint.start_thumb_drag(time_stamp)
            })
            .expect("live ScrollBarConstraint occurrence");
        true
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        self.constraint
            .with_downcast_mut::<ScrollBarConstraint, _>(ScrollBarConstraint::end_thumb_drag)
            .expect("live ScrollBarConstraint occurrence");
        true
    }
    fn hittable(&self) -> Option<CoreHandle> {
        Some(self.hittable.clone())
    }
}

pub struct TrackDraggableProxy {
    constraint: CoreHandle,
    hittable: CoreHandle,
}
impl TrackDraggableProxy {
    pub fn new(constraint: CoreHandle, hittable: CoreHandle) -> Self {
        Self {
            constraint,
            hittable,
        }
    }
}
impl DraggableProxy for TrackDraggableProxy {
    fn start_drag(&mut self, mouse_position: Vec2D, _time_stamp: f32) -> bool {
        self.constraint
            .with_downcast_mut::<ScrollBarConstraint, _>(|constraint| {
                constraint.hit_track(mouse_position)
            })
            .expect("live ScrollBarConstraint occurrence");
        true
    }
    fn drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        true
    }
    fn end_drag(&mut self, _mouse_position: Vec2D, _time_stamp: f32) -> bool {
        true
    }
    fn hittable(&self) -> Option<CoreHandle> {
        Some(self.hittable.clone())
    }
}
