//! Per-pointer state owned by one C++ `ListenerGroup` occurrence.

use crate::state_machine::{RuntimeListenerType, StateMachineEventContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListenerGroupKind {
    Authored { listener_index: usize },
    Draggable { proxy_index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerPhase {
    Out,
    Down,
    Clicked,
    Disabled,
}

#[derive(Debug, Clone)]
struct PointerData {
    pointer_id: i32,
    current_hovered: bool,
    previous_hovered: bool,
    phase: PointerPhase,
    suppress_click: bool,
    previous_position: (f32, f32),
    captured_event_context: Option<StateMachineEventContext>,
}

impl PointerData {
    fn new(pointer_id: i32) -> Self {
        Self {
            pointer_id,
            current_hovered: false,
            previous_hovered: false,
            phase: PointerPhase::Out,
            suppress_click: false,
            previous_position: (0.0, 0.0),
            captured_event_context: None,
        }
    }

    fn reset_for_pointer(&mut self, pointer_id: i32) {
        *self = Self::new(pointer_id);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ListenerPointerState {
    pub(crate) current_hovered: bool,
    pub(crate) previous_hovered: bool,
    pub(crate) phase_was_down: bool,
    pub(crate) phase_is_down: bool,
    pub(crate) clicked: bool,
    pub(crate) drag_ended: bool,
    pub(crate) previous_position: (f32, f32),
}

/// Direct Rust owner for pinned C++ `src/listener_group.cpp`.
#[derive(Debug, Clone)]
pub(crate) struct ListenerGroup {
    pub(crate) kind: ListenerGroupKind,
    pub(crate) is_consumed: bool,
    pointer_data: Vec<PointerData>,
    pointer_data_pool: Vec<PointerData>,
    has_dragged: bool,
}

impl ListenerGroup {
    pub(crate) fn authored(listener_index: usize) -> Self {
        Self::new(ListenerGroupKind::Authored { listener_index })
    }

    pub(crate) fn draggable(proxy_index: usize) -> Self {
        Self::new(ListenerGroupKind::Draggable { proxy_index })
    }

    fn new(kind: ListenerGroupKind) -> Self {
        Self {
            kind,
            is_consumed: false,
            pointer_data: Vec::new(),
            pointer_data_pool: Vec::new(),
            has_dragged: false,
        }
    }

    fn pointer_data_index(&self, pointer_id: i32) -> Option<usize> {
        self.pointer_data
            .iter()
            .position(|data| data.pointer_id == pointer_id)
    }

    fn ensure_pointer_data(&mut self, pointer_id: i32) -> &mut PointerData {
        if let Some(index) = self.pointer_data_index(pointer_id) {
            return &mut self.pointer_data[index];
        }
        let data = self.pointer_data_pool.pop().map_or_else(
            || PointerData::new(pointer_id),
            |mut data| {
                data.reset_for_pointer(pointer_id);
                data
            },
        );
        self.pointer_data.push(data);
        self.pointer_data
            .last_mut()
            .expect("the pointer record was just appended")
    }

    /// Promote current hover to previous hover, clear the new current hit,
    /// and return Clicked to Out. Disabled records remain untouched.
    pub(crate) fn reset(&mut self, pointer_id: i32) {
        self.ensure_pointer_data(pointer_id);
        let index = self
            .pointer_data_index(pointer_id)
            .expect("the pointer record was just ensured");
        if self.pointer_data[index].phase == PointerPhase::Disabled {
            return;
        }
        self.is_consumed = false;
        let data = &mut self.pointer_data[index];
        data.previous_hovered = data.current_hovered;
        data.current_hovered = false;
        data.suppress_click = false;
        if data.phase == PointerPhase::Clicked {
            data.phase = PointerPhase::Out;
        }
    }

    pub(crate) fn hover(&mut self, pointer_id: i32) {
        self.ensure_pointer_data(pointer_id).current_hovered = true;
    }

    pub(crate) fn enable(&mut self, pointer_id: i32) {
        self.ensure_pointer_data(pointer_id).phase = PointerPhase::Out;
    }

    pub(crate) fn disable(&mut self, pointer_id: i32) {
        self.ensure_pointer_data(pointer_id).phase = PointerPhase::Disabled;
        self.is_consumed = true;
    }

    pub(crate) fn disabled(&self, pointer_id: i32) -> bool {
        self.pointer_data
            .iter()
            .find(|data| data.pointer_id == pointer_id)
            .is_some_and(|data| data.phase == PointerPhase::Disabled)
    }

    pub(crate) fn process(
        &mut self,
        pointer_id: i32,
        position: (f32, f32),
        can_hit: bool,
        is_down: bool,
        is_up: bool,
    ) -> ListenerPointerState {
        let had_dragged = self.has_dragged;
        let data = self.ensure_pointer_data(pointer_id);
        if !can_hit {
            data.current_hovered = false;
        }
        if !data.previous_hovered && data.current_hovered {
            data.previous_position = position;
        }
        let phase_was_down = data.phase == PointerPhase::Down;
        if data.current_hovered {
            if is_down {
                data.phase = PointerPhase::Down;
            } else if is_up && phase_was_down {
                data.phase = PointerPhase::Clicked;
            }
        } else if is_down || is_up {
            data.phase = PointerPhase::Out;
        }
        let current_hovered = data.current_hovered;
        let previous_hovered = data.previous_hovered;
        let phase_is_down = data.phase == PointerPhase::Down;
        let clicked = data.phase == PointerPhase::Clicked && !data.suppress_click;
        let previous_position = data.previous_position;
        let drag_ended = phase_was_down
            && matches!(data.phase, PointerPhase::Clicked | PointerPhase::Out)
            && had_dragged;
        if drag_ended {
            self.has_dragged = false;
        }
        ListenerPointerState {
            current_hovered,
            previous_hovered,
            phase_was_down,
            phase_is_down,
            clicked,
            drag_ended,
            previous_position,
        }
    }

    pub(crate) fn record_position(&mut self, pointer_id: i32, position: (f32, f32)) {
        self.ensure_pointer_data(pointer_id).previous_position = position;
    }

    pub(crate) fn previous_position(&self, pointer_id: i32) -> Option<(f32, f32)> {
        self.pointer_data
            .iter()
            .find(|data| data.pointer_id == pointer_id)
            .map(|data| data.previous_position)
    }

    pub(crate) fn mark_dragged(&mut self) {
        self.has_dragged = true;
    }

    pub(crate) fn has_dragged(&self) -> bool {
        self.has_dragged
    }

    pub(crate) fn suppress_click_once(&mut self, pointer_id: i32) {
        self.ensure_pointer_data(pointer_id).suppress_click = true;
    }

    pub(crate) fn begin_capture(
        &mut self,
        pointer_id: i32,
        event_context: Option<&StateMachineEventContext>,
    ) {
        self.ensure_pointer_data(pointer_id).captured_event_context = event_context.cloned();
    }

    pub(crate) fn captured_event_context(
        &self,
        pointer_id: i32,
    ) -> Option<&StateMachineEventContext> {
        self.pointer_data
            .iter()
            .find(|data| data.pointer_id == pointer_id)
            .and_then(|data| data.captured_event_context.as_ref())
    }

    pub(crate) fn phase_is_down(&self, pointer_id: i32) -> bool {
        self.pointer_data
            .iter()
            .find(|data| data.pointer_id == pointer_id)
            .is_some_and(|data| data.phase == PointerPhase::Down)
    }

    pub(crate) fn release_event(&mut self, pointer_id: i32) {
        let Some(index) = self.pointer_data_index(pointer_id) else {
            return;
        };
        let mut data = self.pointer_data.remove(index);
        data.reset_for_pointer(0);
        self.pointer_data_pool.push(data);
    }
}

/// C++ selects one action by overwriting in hover, click/direct, drag order.
pub(crate) fn select_listener_action(
    hover: Option<RuntimeListenerType>,
    click: bool,
    direct: Option<RuntimeListenerType>,
    drag: bool,
) -> Option<RuntimeListenerType> {
    let mut action = hover;
    if click {
        action = Some(RuntimeListenerType::Click);
    } else if action.is_none() {
        action = direct;
    }
    if drag {
        action = Some(RuntimeListenerType::Drag);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_records_are_group_owned_and_released_to_the_pool() {
        let mut group = ListenerGroup::authored(3);
        group.reset(9);
        group.hover(9);
        let down = group.process(9, (4.0, 5.0), true, true, false);
        assert!(down.current_hovered);
        assert_eq!(down.previous_position, (4.0, 5.0));
        group.record_position(9, (6.0, 7.0));
        group.is_consumed = true;
        group.mark_dragged();
        group.release_event(9);
        assert_eq!(group.previous_position(9), None);
        assert!(group.is_consumed, "release is pointer-local");
        assert!(group.has_dragged, "release does not clear group drag state");
        group.reset(11);
        assert_eq!(group.pointer_data.len(), 1);
        assert_eq!(group.pointer_data_pool.len(), 0);
    }

    #[test]
    fn click_and_drag_phases_follow_reset_process_order() {
        let mut group = ListenerGroup::authored(0);
        group.reset(1);
        group.hover(1);
        group.process(1, (0.0, 0.0), true, true, false);
        group.mark_dragged();
        group.reset(1);
        group.hover(1);
        let up = group.process(1, (1.0, 0.0), true, false, true);
        assert!(up.phase_was_down);
        assert!(up.clicked);
        assert!(up.drag_ended);
        group.reset(1);
        let next = group.process(1, (1.0, 0.0), true, false, false);
        assert!(!next.clicked);

        group.reset(2);
        group.hover(2);
        group.process(2, (0.0, 0.0), true, true, false);
        group.reset(2);
        group.process(2, (1.0, 0.0), true, true, false);
        group.reset(2);
        group.hover(2);
        let outside_cancelled = group.process(2, (1.0, 0.0), true, false, true);
        assert!(!outside_cancelled.clicked);
    }

    #[test]
    fn disabled_pointer_state_survives_reset_until_enable() {
        let mut group = ListenerGroup::authored(0);
        group.disable(7);
        group.reset(7);
        assert!(group.disabled(7));
        assert!(group.is_consumed);
        group.enable(7);
        group.reset(7);
        assert!(!group.disabled(7));
        assert!(!group.is_consumed);
    }

    #[test]
    fn pointer_history_preserves_cpp_nonfinite_payload_bits() {
        let mut group = ListenerGroup::authored(0);
        group.reset(77);
        group.record_position(77, (0.0, f32::NAN));
        let position = group.previous_position(77).expect("group pointer record");
        assert_eq!(position.0.to_bits(), 0.0_f32.to_bits());
        assert!(position.1.is_nan());
    }

    #[test]
    fn listener_action_selection_matches_cpp_overwrite_precedence() {
        assert_eq!(
            select_listener_action(
                Some(RuntimeListenerType::Enter),
                true,
                Some(RuntimeListenerType::Up),
                false,
            ),
            Some(RuntimeListenerType::Click)
        );
        assert_eq!(
            select_listener_action(
                Some(RuntimeListenerType::Enter),
                true,
                Some(RuntimeListenerType::Move),
                true,
            ),
            Some(RuntimeListenerType::Drag)
        );
        assert_eq!(
            select_listener_action(
                Some(RuntimeListenerType::Enter),
                false,
                Some(RuntimeListenerType::Move),
                false,
            ),
            Some(RuntimeListenerType::Enter)
        );
    }
}
