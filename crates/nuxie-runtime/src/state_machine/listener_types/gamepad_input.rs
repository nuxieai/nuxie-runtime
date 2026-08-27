use nuxie_binary::RuntimeObject;

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
}
