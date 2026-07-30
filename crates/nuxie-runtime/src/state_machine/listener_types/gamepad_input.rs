use nuxie_binary::RuntimeObject;

const GAMEPAD_INPUT_KIND_BUTTON: u32 = 0;
const GAMEPAD_INPUT_KIND_AXIS: u32 = 1;
const GAMEPAD_INPUT_KIND_CONNECTED: u32 = 2;
const GAMEPAD_INPUT_KIND_DISCONNECTED: u32 = 3;
const GAMEPAD_INPUT_MAPPING_STANDARD: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeGamepadInputEvent {
    Connected,
    Disconnected,
    Button {
        index: u32,
        value: f32,
        standard_intent: Option<u32>,
    },
    Axis {
        index: u32,
        standard_intent: Option<u32>,
    },
}

/// Authored gamepad constraint owned by one ListenerInputTypeGamepad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeGamepadInput {
    pub(crate) global_id: u32,
    pub(super) kind: u32,
    pub(super) mapping: u32,
    pub(super) input_index: u32,
    pub(super) button_phase: u32,
}

impl RuntimeGamepadInput {
    pub(super) fn from_imported(object: &RuntimeObject) -> Self {
        Self {
            global_id: object.id,
            kind: object.uint_property("kind").unwrap_or(0) as u32,
            mapping: object.uint_property("mapping").unwrap_or(0) as u32,
            input_index: object.uint_property("inputIndex").unwrap_or(0) as u32,
            button_phase: object.uint_property("buttonPhase").unwrap_or(1) as u32,
        }
    }

    pub(super) fn matches(self, event: RuntimeGamepadInputEvent) -> bool {
        match (self.kind, event) {
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
                if !self.index_matches(index, standard_intent) {
                    return false;
                }
                super::listener_input_type_gamepad::gamepad_button_phase_matches(
                    self.button_phase,
                    value >= 0.5,
                )
            }
            (
                GAMEPAD_INPUT_KIND_AXIS,
                RuntimeGamepadInputEvent::Axis {
                    index,
                    standard_intent,
                },
            ) => self.index_matches(index, standard_intent),
            _ => false,
        }
    }

    fn index_matches(self, raw_index: u32, standard_intent: Option<u32>) -> bool {
        if self.mapping == GAMEPAD_INPUT_MAPPING_STANDARD {
            standard_intent == Some(self.input_index)
        } else {
            raw_index == self.input_index
        }
    }
}
