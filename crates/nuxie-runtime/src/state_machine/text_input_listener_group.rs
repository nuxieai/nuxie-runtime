//! TextInput pointer bridge ported from
//! `src/animation/text_input_listener_group.cpp`.

use super::RuntimeListenerType;
use crate::ArtboardInstance;
use crate::listener_group::ListenerPointerState;
use std::sync::OnceLock;
use std::time::Instant;

const MULTI_CLICK_INTERVAL_US: i64 = 500_000;
const MULTI_CLICK_DISTANCE: f32 = 16.0;

fn click_timestamp_micros(timestamp_seconds: f32) -> i64 {
    if crate::math::random::runtime_deterministic_mode() {
        return (timestamp_seconds * 1_000_000.0) as i64;
    }
    static CLOCK_ORIGIN: OnceLock<Instant> = OnceLock::new();
    CLOCK_ORIGIN
        .get_or_init(Instant::now)
        .elapsed()
        .as_micros()
        .min(i64::MAX as u128) as i64
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTextInputListenerGroup {
    pub(crate) text_input_local_id: usize,
    is_dragging: bool,
    click_count: u8,
    last_click_time_us: i64,
    last_click_position: (f32, f32),
}

impl RuntimeTextInputListenerGroup {
    pub(crate) fn new(text_input_local_id: usize) -> Self {
        Self {
            text_input_local_id,
            is_dragging: false,
            click_count: 0,
            last_click_time_us: 0,
            last_click_position: (0.0, 0.0),
        }
    }

    pub(crate) fn process_event(
        &mut self,
        artboard: &mut ArtboardInstance,
        pointer: ListenerPointerState,
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        timestamp_seconds: f32,
    ) -> TextInputEventResult {
        if !pointer.phase_was_down && pointer.phase_is_down {
            let now = click_timestamp_micros(timestamp_seconds);
            let dt = now.saturating_sub(self.last_click_time_us);
            let dx = position.0 - self.last_click_position.0;
            let dy = position.1 - self.last_click_position.1;
            let distance = (dx * dx + dy * dy).sqrt();
            self.click_count = if (0..=MULTI_CLICK_INTERVAL_US).contains(&dt)
                && distance <= MULTI_CLICK_DISTANCE
            {
                if self.click_count >= 3 {
                    1
                } else {
                    self.click_count.saturating_add(1)
                }
            } else {
                1
            };
            self.last_click_time_us = now;
            self.last_click_position = position;
            artboard.text_input_start_drag(self.text_input_local_id, position);
            self.is_dragging = true;
            if self.click_count == 2 {
                artboard.text_input_select_word(self.text_input_local_id);
            } else if self.click_count == 3 {
                artboard.text_input_select_line(self.text_input_local_id);
            }
            return TextInputEventResult {
                blocks: true,
                focus_requested: true,
            };
        }
        if hit_type == RuntimeListenerType::Move && pointer.phase_is_down && self.is_dragging {
            artboard.text_input_drag(self.text_input_local_id, position);
            return TextInputEventResult {
                blocks: true,
                focus_requested: false,
            };
        }
        if pointer.phase_was_down && !pointer.phase_is_down && self.is_dragging {
            artboard.text_input_end_drag(self.text_input_local_id);
            self.is_dragging = false;
        }
        TextInputEventResult {
            blocks: false,
            focus_requested: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextInputEventResult {
    pub(crate) blocks: bool,
    pub(crate) focus_requested: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_multi_click_constants_and_initial_state_are_pinned() {
        let group = RuntimeTextInputListenerGroup::new(7);
        assert_eq!(group.text_input_local_id, 7);
        assert!(!group.is_dragging);
        assert_eq!(MULTI_CLICK_INTERVAL_US, 500_000);
        assert_eq!(MULTI_CLICK_DISTANCE, 16.0);
    }
}
