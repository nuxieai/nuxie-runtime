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
    cell::Cell,
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct TextInputListenerGroup {
    base: ListenerGroup,
    text_input: CoreHandle,
    is_dragging: Cell<bool>,
    click_count: Cell<i32>,
    last_click_time: Cell<i64>,
    last_click_position: Cell<Vec2D>,
}

impl TextInputListenerGroup {
    pub fn new(text_input: CoreHandle) -> Self {
        Self {
            base: ListenerGroup::new_optional(None),
            text_input,
            is_dragging: Cell::new(false),
            click_count: Cell::new(0),
            last_click_time: Cell::new(0),
            last_click_position: Cell::new(Vec2D::new(0.0, 0.0)),
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
    fn reset(&self, id: i32) {
        self.base.reset(id);
    }
    fn release_event(&self, id: i32) {
        self.base.release_event(id);
    }
    fn hover(&self, id: i32) {
        self.base.hover(id);
    }
    fn enable(&self, id: i32) {
        self.base.enable(id);
    }
    fn disable(&self, id: i32) {
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
        &self,
        _component: &RuntimeDrawableOccurrence,
        position: Vec2D,
        pointer_id: i32,
        event: ListenerType,
        can_hit: bool,
        timestamp: f32,
        machine: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let pointer = self.base.pointer_data(pointer_id);
        let previous = pointer.phase.get();
        if !can_hit && pointer.is_hovered.get() {
            pointer.is_hovered.set(false);
        }
        let hovered = can_hit && pointer.is_hovered.get();
        if hovered {
            if event == ListenerType::Down {
                pointer.phase.set(GestureClickPhase::Down);
            } else if event == ListenerType::Up && pointer.phase.get() == GestureClickPhase::Down {
                pointer.phase.set(GestureClickPhase::Clicked);
            }
        } else if matches!(event, ListenerType::Down | ListenerType::Up) {
            pointer.phase.set(GestureClickPhase::Out);
        }
        let phase = pointer.phase.get();
        if previous != GestureClickPhase::Down && phase == GestureClickPhase::Down {
            let now = Self::now_micros(timestamp);
            let dt = now - self.last_click_time.get();
            let distance = Vec2D::distance(position, self.last_click_position.get());
            self.click_count
                .set(if dt >= 0 && dt <= 500_000 && distance <= 16.0 {
                    if self.click_count.get() >= 3 {
                        1
                    } else {
                        self.click_count.get() + 1
                    }
                } else {
                    1
                });
            self.last_click_time.set(now);
            self.last_click_position.set(position);
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .start_drag(position)
            });
            self.is_dragging.set(true);
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
                if self.click_count.get() == 2 {
                    input.select_word();
                } else if self.click_count.get() == 3 {
                    input.select_line();
                }
            });
            ProcessEventResult::Scroll
        } else if event == ListenerType::Move
            && phase == GestureClickPhase::Down
            && self.is_dragging.get()
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
                && self.is_dragging.get()
            {
                self.text_input.with_mut(|input| {
                    input
                        .as_text_input_mut()
                        .expect("text hit owner remains TextInput")
                        .end_drag(position)
                });
                self.is_dragging.set(false);
            }
            ProcessEventResult::None
        }
    }
}
