use crate::mechanical_port::source::{listener_type::ListenerType, math::vec2d::Vec2D};
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GestureClickPhase {
    None,
    Down,
    Clicked,
    Out,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessEventResult {
    None,
    Scroll,
}
pub trait TextInputListenerHost {
    fn deterministic_mode(&self) -> bool;
    fn phase(&self, pointer: i32) -> GestureClickPhase;
    fn set_phase(&mut self, pointer: i32, phase: GestureClickPhase);
    fn hovered(&self, pointer: i32) -> bool;
    fn set_hovered(&mut self, pointer: i32, value: bool);
    fn start_drag(&mut self, position: Vec2D);
    fn drag(&mut self, position: Vec2D);
    fn end_drag(&mut self, position: Vec2D);
    fn focus_text_input(&mut self);
    fn select_word(&mut self);
    fn select_line(&mut self);
}
pub struct TextInputListenerGroup {
    is_dragging: bool,
    click_count: i32,
    last_click_time: i64,
    last_click_position: Vec2D,
}
impl Default for TextInputListenerGroup {
    fn default() -> Self {
        Self {
            is_dragging: false,
            click_count: 0,
            last_click_time: 0,
            last_click_position: Vec2D::new(0.0, 0.0),
        }
    }
}
impl TextInputListenerGroup {
    fn now_micros(host: &dyn TextInputListenerHost, timestamp: f32) -> i64 {
        if host.deterministic_mode() {
            (timestamp * 1_000_000.0) as i64
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64
        }
    }
    pub fn can_early_out(&self) -> bool {
        false
    }
    pub fn needs_down_listener(&self) -> bool {
        true
    }
    pub fn needs_up_listener(&self) -> bool {
        true
    }
    pub fn process_event(
        &mut self,
        host: &mut dyn TextInputListenerHost,
        position: Vec2D,
        pointer: i32,
        event: ListenerType,
        can_hit: bool,
        timestamp: f32,
    ) -> ProcessEventResult {
        let previous = host.phase(pointer);
        if !can_hit && host.hovered(pointer) {
            host.set_hovered(pointer, false);
        }
        let hovered = can_hit && host.hovered(pointer);
        if hovered {
            if event == ListenerType::Down {
                host.set_phase(pointer, GestureClickPhase::Down);
            } else if event == ListenerType::Up && previous == GestureClickPhase::Down {
                host.set_phase(pointer, GestureClickPhase::Clicked);
            }
        } else if event == ListenerType::Down || event == ListenerType::Up {
            host.set_phase(pointer, GestureClickPhase::Out);
        }
        let phase = host.phase(pointer);
        if previous != GestureClickPhase::Down && phase == GestureClickPhase::Down {
            let now = Self::now_micros(host, timestamp);
            let dt = now - self.last_click_time;
            let distance = Vec2D::distance(position, self.last_click_position);
            self.click_count = if dt >= 0 && dt <= 500_000 && distance <= 16.0 {
                if self.click_count >= 3 {
                    1
                } else {
                    self.click_count + 1
                }
            } else {
                1
            };
            self.last_click_time = now;
            self.last_click_position = position;
            host.start_drag(position);
            self.is_dragging = true;
            host.focus_text_input();
            if self.click_count == 2 {
                host.select_word();
            } else if self.click_count == 3 {
                host.select_line();
            }
            return ProcessEventResult::Scroll;
        } else if event == ListenerType::Move
            && phase == GestureClickPhase::Down
            && self.is_dragging
        {
            host.drag(position);
            return ProcessEventResult::Scroll;
        } else if previous == GestureClickPhase::Down
            && (phase == GestureClickPhase::Clicked || phase == GestureClickPhase::Out)
            && self.is_dragging
        {
            host.end_drag(position);
            self.is_dragging = false;
        }
        ProcessEventResult::None
    }
}
