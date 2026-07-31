use nuxie_binary::RuntimeObject;

/// Authored keyboard constraint owned by one ListenerInputTypeKeyboard.
///
/// C++ retains the concrete KeyboardInput objects under their typed importer.
/// Rust retains the same source identity and serialized values; mutable
/// listener-group state is occurrence-owned elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeKeyboardInput {
    pub(crate) global_id: u32,
    pub(super) key_type: u32,
    pub(super) key_phase: u32,
    pub(super) modifiers: u32,
}

impl RuntimeKeyboardInput {
    pub(super) fn from_imported(object: &RuntimeObject) -> Self {
        Self {
            global_id: object.id,
            key_type: object.uint_property("keyType").unwrap_or(u32::MAX as u64) as u32,
            key_phase: object.uint_property("keyPhase").unwrap_or(0) as u32,
            modifiers: object.uint_property("modifiers").unwrap_or(0) as u32,
        }
    }

    pub(super) fn matches(
        self,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        if self.key_type != u32::MAX && self.key_type != key {
            return false;
        }
        if self.modifiers != modifiers {
            return false;
        }
        super::listener_input_type_keyboard::key_phase_matches(
            self.key_phase,
            is_pressed,
            is_repeat,
        )
    }
}
