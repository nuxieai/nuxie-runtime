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
use std::{cell::Cell, sync::atomic::Ordering};

pub struct TextInputListenerGroup {
    base: ListenerGroup,
    text_input: CoreHandle,
    dragging_pointer_id: Cell<i32>,
    click_count: Cell<i32>,
    last_click_time: Cell<i64>,
    last_click_position: Cell<Vec2D>,
    /// The field is CSS content (docs/CSS_LAYOUT.md, 4.5): unfocused, a
    /// press on it is a tap that focuses it on release, and its drag goes
    /// to what is around it; focused, it takes the drag to select.
    in_css: Cell<bool>,
    /// The pointers pressing the unfocused CSS field, each waiting to be a
    /// tap.
    pending_taps: std::cell::RefCell<Vec<i32>>,
}

impl TextInputListenerGroup {
    pub fn new(text_input: CoreHandle) -> Self {
        Self {
            base: ListenerGroup::new_optional(None),
            text_input,
            dragging_pointer_id: Cell::new(-1),
            click_count: Cell::new(0),
            last_click_time: Cell::new(0),
            last_click_position: Cell::new(Vec2D::new(0.0, 0.0)),
            in_css: Cell::new(false),
            pending_taps: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Ends `pointer_id`'s pending tap: whether it had one.
    fn settle_tap(&self, pointer_id: i32) -> bool {
        let mut taps = self.pending_taps.borrow_mut();
        let pending = taps.contains(&pointer_id);
        taps.retain(|&id| id != pointer_id);
        pending
    }

    fn is_focused(&self) -> bool {
        self.text_input
            .with(|input| input.as_text_input().map(|input| input.is_focused()))
            .flatten()
            .unwrap_or(false)
    }

    /// Counts a press toward a double or triple click.
    fn count_click(&self, position: Vec2D, timestamp: f32) {
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
    }

    fn focus(&self, machine: &StateMachineInstance) {
        if let Some(manager) = machine.focus_manager() {
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
                    manager.with_focus_manager_mut(|manager| manager.set_focus(node));
                    break;
                }
            }
        }
    }

    fn now_micros(timestamp: f32) -> i64 {
        if DETERMINISTIC_MODE.load(Ordering::Relaxed) {
            (timestamp * 1_000_000.0) as i64
        } else {
            crate::mechanical_port::source::engine_clock::now_micros()
        }
    }
}

impl ListenerGroupBehavior for TextInputListenerGroup {
    fn is_pointer_disabled(&self, id: i32) -> bool {
        self.base.is_pointer_disabled(id)
    }
    fn is_pointer_hovered(&self, id: i32) -> bool {
        self.base.is_pointer_hovered(id)
    }
    fn tracked_pointer_ids(&self) -> Vec<i32> {
        self.base.tracked_pointer_ids()
    }
    fn cancel_pointer(&self, id: i32, position: Vec2D, timestamp: f32) -> bool {
        self.settle_tap(id);
        let was_dragging = self.base.cancel_pointer(id, position, timestamp);
        if self.dragging_pointer_id.get() == id {
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .end_drag(position)
            });
            self.dragging_pointer_id.set(-1);
        }
        was_dragging
    }
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
        // A scroll that starts under the press cancels its tap.
        self.settle_tap(id);
        self.base.disable(id);
    }
    fn is_consumed(&self) -> bool {
        self.base.is_consumed()
    }
    fn can_early_out(&self, _component: &Component) -> bool {
        false
    }
    fn mark_in_css(&self) {
        self.in_css.set(true);
    }
    fn takes_drag(&self) -> bool {
        !self.in_css.get() || self.is_focused()
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
        button: crate::source::pointer_button::PointerButton,
        can_hit: bool,
        timestamp: f32,
        machine: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        if matches!(event, ListenerType::Down | ListenerType::Up)
            && button != crate::source::pointer_button::PointerButton::Primary
        {
            return ProcessEventResult::None;
        }
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
            if self.in_css.get() && !self.is_focused() {
                // A tap focuses the field on release; a drag scrolls what
                // is around it (docs/CSS_LAYOUT.md, 4.5).
                let mut taps = self.pending_taps.borrow_mut();
                if !taps.contains(&pointer_id) {
                    taps.push(pointer_id);
                }
                return ProcessEventResult::None;
            }
            self.count_click(position, timestamp);
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .start_drag(position)
            });
            self.dragging_pointer_id.set(pointer_id);
            if self.in_css.get() {
                // A focused CSS field's drag selects: it is the innermost
                // drag handler (docs/CSS_LAYOUT.md, 4.5).
                machine.claim_css_drag(pointer_id, self.identity(), position, timestamp);
            }
            self.focus(machine);
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
            && self.dragging_pointer_id.get() == pointer_id
        {
            self.text_input.with_mut(|input| {
                input
                    .as_text_input_mut()
                    .expect("text hit owner remains TextInput")
                    .drag(position)
            });
            ProcessEventResult::Scroll
        } else {
            // Only the press's end settles the tap: moves below the scroll
            // threshold keep it.
            if previous == GestureClickPhase::Down
                && matches!(phase, GestureClickPhase::Clicked | GestureClickPhase::Out)
                && self.settle_tap(pointer_id)
            {
                if phase == GestureClickPhase::Clicked {
                    // The tap: focus the field with its caret where it was
                    // released.
                    self.count_click(position, timestamp);
                    self.text_input.with_mut(|input| {
                        let input = input
                            .as_text_input_mut()
                            .expect("text hit owner remains TextInput");
                        input.start_drag(position);
                        input.end_drag(position);
                    });
                    self.focus(machine);
                }
                return ProcessEventResult::None;
            }
            if previous == GestureClickPhase::Down
                && matches!(phase, GestureClickPhase::Clicked | GestureClickPhase::Out)
                && self.dragging_pointer_id.get() == pointer_id
            {
                self.text_input.with_mut(|input| {
                    input
                        .as_text_input_mut()
                        .expect("text hit owner remains TextInput")
                        .end_drag(position)
                });
                self.dragging_pointer_id.set(-1);
            }
            ProcessEventResult::None
        }
    }
}

#[cfg(test)]
mod clock_contract_tests {
    use super::TextInputListenerGroup;
    use crate::mechanical_port::source::engine_clock::with_test_clock;

    #[test]
    fn text_input_reads_the_shared_monotonic_clock_without_a_frame() {
        // The embedder clock can move while playback is paused. The same
        // engine clock boundary supplies performance.now on browser Wasm.
        with_test_clock(
            || 1_000_000,
            || {
                assert_eq!(TextInputListenerGroup::now_micros(0.0), 1_000_000);
            },
        );
        with_test_clock(
            || 1_750_000,
            || {
                assert_eq!(TextInputListenerGroup::now_micros(0.0), 1_750_000);
            },
        );
    }
}
