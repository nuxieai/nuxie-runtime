use super::gamepad_input::{RuntimeGamepadInput, RuntimeGamepadInputEvent};
use nuxie_binary::RuntimeObject;

const GAMEPAD_BUTTON_PHASE_DOWN: u32 = 1;
const GAMEPAD_BUTTON_PHASE_UP: u32 = 2;
const GAMEPAD_BUTTON_PHASE_ALL: u32 = GAMEPAD_BUTTON_PHASE_DOWN | GAMEPAD_BUTTON_PHASE_UP;
const GAMEPAD_INPUT_KIND_BUTTON: u32 = 0;
const GAMEPAD_INPUT_KIND_AXIS: u32 = 1;
const GAMEPAD_INPUT_KIND_CONNECTED: u32 = 2;
const GAMEPAD_INPUT_KIND_DISCONNECTED: u32 = 3;
const GAMEPAD_INPUT_MAPPING_STANDARD: u32 = 0;

/// Authored ListenerInputTypeGamepad definition shared by occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeListenerInputTypeGamepad {
    pub(crate) global_id: u32,
    gamepad_inputs: Vec<RuntimeGamepadInput>,
}

impl RuntimeListenerInputTypeGamepad {
    pub(in crate::state_machine) fn from_imported(
        input_type: &RuntimeObject,
        inputs: &[&RuntimeObject],
    ) -> Self {
        let mut owner = Self {
            global_id: input_type.id,
            gamepad_inputs: Vec::new(),
        };
        for input in inputs
            .iter()
            .filter(|input| input.type_name == "GamepadInput")
        {
            owner.add_gamepad_input(RuntimeGamepadInput::from_imported(input));
        }
        owner
    }

    pub(crate) fn gamepad_input_count(&self) -> usize {
        self.gamepad_inputs.len()
    }

    pub(crate) fn gamepad_input(&self, index: usize) -> Option<&RuntimeGamepadInput> {
        self.gamepad_inputs.get(index)
    }

    fn add_gamepad_input(&mut self, input: RuntimeGamepadInput) {
        if self
            .gamepad_inputs
            .iter()
            .any(|existing| existing.global_id == input.global_id)
        {
            return;
        }
        self.gamepad_inputs.push(input);
    }

    #[cfg(test)]
    pub(in crate::state_machine) fn catch_all_for_test(global_id: u32) -> Self {
        Self {
            global_id,
            gamepad_inputs: Vec::new(),
        }
    }

    pub(crate) fn constraints_met(input_types: &[Self], event: RuntimeGamepadInputEvent) -> bool {
        for input_type in input_types {
            if input_type.gamepad_inputs.is_empty() {
                return true;
            }
            if input_type
                .gamepad_inputs
                .iter()
                .any(|input| gamepad_input_matches(input, event))
            {
                return true;
            }
        }
        false
    }
}

fn gamepad_input_matches(
    input: &RuntimeGamepadInput,
    event: RuntimeGamepadInputEvent,
) -> bool {
    match (input.kind, event) {
        (GAMEPAD_INPUT_KIND_CONNECTED, RuntimeGamepadInputEvent::Connected)
        | (GAMEPAD_INPUT_KIND_DISCONNECTED, RuntimeGamepadInputEvent::Disconnected) => true,
        (
            GAMEPAD_INPUT_KIND_BUTTON,
            RuntimeGamepadInputEvent::Button {
                index,
                value,
                standard_intent,
            },
        ) => {
            if !gamepad_input_index_matches(input, index, standard_intent) {
                return false;
            }
            gamepad_button_phase_matches(input.button_phase, value >= 0.5)
        }
        (
            GAMEPAD_INPUT_KIND_AXIS,
            RuntimeGamepadInputEvent::Axis {
                index,
                standard_intent,
            },
        ) => gamepad_input_index_matches(input, index, standard_intent),
        _ => false,
    }
}

fn gamepad_input_index_matches(
    input: &RuntimeGamepadInput,
    raw_index: u32,
    standard_intent: Option<u32>,
) -> bool {
    if input.mapping == GAMEPAD_INPUT_MAPPING_STANDARD {
        standard_intent == Some(input.input_index)
    } else {
        raw_index == input.input_index
    }
}

pub(super) fn gamepad_button_phase_matches(phase: u32, is_pressed: bool) -> bool {
    let mask = phase & GAMEPAD_BUTTON_PHASE_ALL;
    if mask == 0 {
        return false;
    }
    if is_pressed {
        return mask & GAMEPAD_BUTTON_PHASE_DOWN != 0;
    }
    mask & GAMEPAD_BUTTON_PHASE_UP != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gamepad_input(
        kind: u32,
        mapping: u32,
        input_index: u32,
        button_phase: u32,
    ) -> RuntimeGamepadInput {
        RuntimeGamepadInput {
            global_id: 1,
            kind,
            mapping,
            input_index,
            button_phase,
        }
    }

    #[test]
    fn button_phase_masks_and_threshold_match_cpp() {
        assert!(!gamepad_button_phase_matches(0, true));
        assert!(gamepad_button_phase_matches(
            GAMEPAD_BUTTON_PHASE_DOWN,
            true,
        ));
        assert!(!gamepad_button_phase_matches(
            GAMEPAD_BUTTON_PHASE_DOWN,
            false,
        ));
        assert!(gamepad_button_phase_matches(GAMEPAD_BUTTON_PHASE_UP, false,));

        let button = gamepad_input(0, 1, 3, GAMEPAD_BUTTON_PHASE_DOWN);
        assert!(gamepad_input_matches(&button, RuntimeGamepadInputEvent::Button {
            index: 3,
            value: 0.5,
            standard_intent: None,
        }));
        assert!(!gamepad_input_matches(&button, RuntimeGamepadInputEvent::Button {
            index: 3,
            value: f32::NAN,
            standard_intent: None,
        }));
    }

    #[test]
    fn standard_mapping_requires_matching_standard_intent() {
        let standard_button = gamepad_input(0, 0, 12, GAMEPAD_BUTTON_PHASE_DOWN);
        assert!(gamepad_input_matches(&standard_button, RuntimeGamepadInputEvent::Button {
            index: 99,
            value: 1.0,
            standard_intent: Some(12),
        }));
        assert!(!gamepad_input_matches(&standard_button, RuntimeGamepadInputEvent::Button {
            index: 12,
            value: 1.0,
            standard_intent: None,
        }));

        let raw_axis = gamepad_input(1, 1, 4, GAMEPAD_BUTTON_PHASE_DOWN);
        assert!(gamepad_input_matches(&raw_axis, RuntimeGamepadInputEvent::Axis {
            index: 4,
            standard_intent: None,
        }));
        assert!(!gamepad_input_matches(&raw_axis, RuntimeGamepadInputEvent::Axis {
            index: 5,
            standard_intent: Some(4),
        }));
    }

    #[test]
    fn connected_disconnected_and_event_kinds_are_exact() {
        assert!(gamepad_input_matches(
            &gamepad_input(2, 0, 0, 1),
            RuntimeGamepadInputEvent::Connected,
        ));
        assert!(gamepad_input_matches(
            &gamepad_input(3, 0, 0, 1),
            RuntimeGamepadInputEvent::Disconnected,
        ));
        assert!(!gamepad_input_matches(
            &gamepad_input(2, 0, 0, 1),
            RuntimeGamepadInputEvent::Disconnected,
        ));
        assert!(!gamepad_input_matches(
            &gamepad_input(99, 0, 0, 1),
            RuntimeGamepadInputEvent::Connected,
        ));
    }

    #[test]
    fn empty_gamepad_type_is_catch_all() {
        let catch_all = RuntimeListenerInputTypeGamepad {
            global_id: 2,
            gamepad_inputs: Vec::new(),
        };
        assert!(RuntimeListenerInputTypeGamepad::constraints_met(
            &[catch_all],
            RuntimeGamepadInputEvent::Disconnected,
        ));
        assert!(!RuntimeListenerInputTypeGamepad::constraints_met(
            &[],
            RuntimeGamepadInputEvent::Disconnected,
        ));
    }
}
