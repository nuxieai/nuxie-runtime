use crate::mechanical_port::source::{
    animation::state_machine_instance::StateMachineInstance,
    component::Component,
    core::CoreHandle,
    drawable::RuntimeDrawableOccurrence,
    file::DETERMINISTIC_MODE,
    focus_data::FocusData,
    gesture_click_phase::GestureClickPhase,
    listener_group::{ListenerGroup, ListenerGroupBehavior},
    listener_type::ListenerType,
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};
use std::{
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct TextInputListenerGroup {
    base: ListenerGroup,
    text_input: CoreHandle,
    is_dragging: bool,
    click_count: i32,
    last_click_time: i64,
    last_click_position: Vec2D,
}

impl TextInputListenerGroup {
    pub fn new(text_input: CoreHandle) -> Self {
        Self {
            base: ListenerGroup::new_optional(None),
            text_input,
            is_dragging: false,
            click_count: 0,
            last_click_time: 0,
            last_click_position: Vec2D::new(0.0, 0.0),
        }
    }

    fn now_micros(timestamp: f32) -> i64 {
        if DETERMINISTIC_MODE.load(Ordering::Relaxed) {
            (timestamp * 1_000_000.0) as i64
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("the host clock is after the epoch")
                .as_micros() as i64
        }
    }
}

impl ListenerGroupBehavior for TextInputListenerGroup {
    fn reset(&mut self, id: i32) {
        self.base.reset(id);
    }
    fn release_event(&mut self, id: i32) {
        self.base.release_event(id);
    }
    fn hover(&mut self, id: i32) {
        self.base.hover(id);
    }
    fn enable(&mut self, id: i32) {
        self.base.enable(id);
    }
    fn disable(&mut self, id: i32) {
        self.base.disable(id);
    }
    fn is_consumed(&self) -> bool {
        self.base.is_consumed()
    }
    fn can_early_out(&self, _component: &Component) -> bool {
        false
    }
    fn needs_down_listener(&self, _component: &Component) -> bool {
        true
    }
    fn needs_up_listener(&self, _component: &Component) -> bool {
        true
    }

    fn process_event(
        &mut self,
        _component: &RuntimeDrawableOccurrence,
        position: Vec2D,
        pointer_id: i32,
        event: ListenerType,
        can_hit: bool,
        timestamp: f32,
        machine: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let pointer = self.base.pointer_data(pointer_id);
        let previous = pointer.phase;
        if !can_hit && pointer.is_hovered {
            pointer.is_hovered = false;
        }
        let hovered = can_hit && pointer.is_hovered;
        if hovered {
            if event == ListenerType::Down {
                pointer.phase = GestureClickPhase::Down;
            } else if event == ListenerType::Up && pointer.phase == GestureClickPhase::Down {
                pointer.phase = GestureClickPhase::Clicked;
            }
        } else if matches!(event, ListenerType::Down | ListenerType::Up) {
            pointer.phase = GestureClickPhase::Out;
        }
        let phase = pointer.phase;
        if previous != GestureClickPhase::Down && phase == GestureClickPhase::Down {
            let now = Self::now_micros(timestamp);
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
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .start_drag(position)
            });
            self.is_dragging = true;
            let children = self
                .text_input
                .with(|input| {
                    input
                        .as_container_component()
                        .map(|container| container.children().to_vec())
                })
                .flatten()
                .expect("TextInput retains its component children");
            for child in children {
                if let Some(node) = child.with_downcast_mut::<FocusData, _>(FocusData::focus_node) {
                    machine
                        .focus_manager()
                        .with_focus_manager_mut(|manager| manager.set_focus(node));
                    break;
                }
            }
            self.text_input.with_mut(|input| {
                let input = input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput");
                if self.click_count == 2 {
                    input.select_word();
                } else if self.click_count == 3 {
                    input.select_line();
                }
            });
            ProcessEventResult::Scroll
        } else if event == ListenerType::Move
            && phase == GestureClickPhase::Down
            && self.is_dragging
        {
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .drag(position)
            });
            ProcessEventResult::Scroll
        } else {
            if previous == GestureClickPhase::Down
                && matches!(phase, GestureClickPhase::Clicked | GestureClickPhase::Out)
                && self.is_dragging
            {
                self.text_input.with_mut(|input| {
                    input
                        .as_text_input_mut()
                        .expect("text hit owner remains TextInput")
                        .end_drag(position)
                });
                self.is_dragging = false;
            }
            ProcessEventResult::None
        }
    }
}
