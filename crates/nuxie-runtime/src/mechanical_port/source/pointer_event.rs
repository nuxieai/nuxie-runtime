use crate::mechanical_port::source::math::vec2d::Vec2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEventType {
    Down,
    Move,
    Up,
}

pub struct PointerEvent {
    pub event_type: PointerEventType,
    pub position: Vec2D,
    pub pointer_index: i32,
}
