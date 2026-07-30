use super::gamepad_input::{RuntimeGamepadInput, RuntimeGamepadInputEvent};
use nuxie_binary::RuntimeObject;

const GAMEPAD_BUTTON_PHASE_DOWN: u32 = 1;
const GAMEPAD_BUTTON_PHASE_UP: u32 = 2;
const GAMEPAD_BUTTON_PHASE_ALL: u32 = GAMEPAD_BUTTON_PHASE_DOWN | GAMEPAD_BUTTON_PHASE_UP;

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
        Self {
            global_id: input_type.id,
            gamepad_inputs: inputs
                .iter()
                .filter(|input| input.type_name == "GamepadInput")
                .map(|input| RuntimeGamepadInput::from_imported(input))
                .collect(),
        }
    }

    pub(crate) fn gamepad_input_count(&self) -> usize {
        self.gamepad_inputs.len()
    }

    pub(crate) fn gamepad_input(&self, index: usize) -> Option<&RuntimeGamepadInput> {
        self.gamepad_inputs.get(index)
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
                .any(|input| input.matches(event))
            {
                return true;
            }
        }
        false
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
        assert!(button.matches(RuntimeGamepadInputEvent::Button {
            index: 3,
            value: 0.5,
            standard_intent: None,
        }));
        assert!(!button.matches(RuntimeGamepadInputEvent::Button {
            index: 3,
            value: f32::NAN,
            standard_intent: None,
        }));
    }

    #[test]
    fn standard_mapping_requires_matching_standard_intent() {
        let standard_button = gamepad_input(0, 0, 12, GAMEPAD_BUTTON_PHASE_DOWN);
        assert!(standard_button.matches(RuntimeGamepadInputEvent::Button {
            index: 99,
            value: 1.0,
            standard_intent: Some(12),
        }));
        assert!(!standard_button.matches(RuntimeGamepadInputEvent::Button {
            index: 12,
            value: 1.0,
            standard_intent: None,
        }));

        let raw_axis = gamepad_input(1, 1, 4, GAMEPAD_BUTTON_PHASE_DOWN);
        assert!(raw_axis.matches(RuntimeGamepadInputEvent::Axis {
            index: 4,
            standard_intent: None,
        }));
        assert!(!raw_axis.matches(RuntimeGamepadInputEvent::Axis {
            index: 5,
            standard_intent: Some(4),
        }));
    }

    #[test]
    fn connected_disconnected_and_event_kinds_are_exact() {
        assert!(gamepad_input(2, 0, 0, 1).matches(RuntimeGamepadInputEvent::Connected));
        assert!(gamepad_input(3, 0, 0, 1).matches(RuntimeGamepadInputEvent::Disconnected));
        assert!(!gamepad_input(2, 0, 0, 1).matches(RuntimeGamepadInputEvent::Disconnected));
        assert!(!gamepad_input(99, 0, 0, 1).matches(RuntimeGamepadInputEvent::Connected));
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
