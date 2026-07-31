use super::keyboard_input::RuntimeKeyboardInput;
use nuxie_binary::RuntimeObject;

const KEY_PHASE_DOWN: u32 = 1;
const KEY_PHASE_REPEAT: u32 = 2;
const KEY_PHASE_UP: u32 = 4;
const KEY_PHASE_ALL: u32 = KEY_PHASE_DOWN | KEY_PHASE_REPEAT | KEY_PHASE_UP;

/// Authored ListenerInputTypeKeyboard definition.
///
/// This is definition state shared by StateMachineInstance occurrences. The
/// KeyboardListenerGroup that registers focused targets is occurrence-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeListenerInputTypeKeyboard {
    pub(crate) global_id: u32,
    keyboard_inputs: Vec<RuntimeKeyboardInput>,
}

impl RuntimeListenerInputTypeKeyboard {
    pub(in crate::state_machine) fn from_imported(
        input_type: &RuntimeObject,
        inputs: &[&RuntimeObject],
    ) -> Self {
        Self {
            global_id: input_type.id,
            keyboard_inputs: inputs
                .iter()
                .filter(|input| input.type_name == "KeyboardInput")
                .map(|input| RuntimeKeyboardInput::from_imported(input))
                .collect(),
        }
    }

    pub(crate) fn keyboard_input_count(&self) -> usize {
        self.keyboard_inputs.len()
    }

    pub(crate) fn keyboard_input(&self, index: usize) -> Option<&RuntimeKeyboardInput> {
        self.keyboard_inputs.get(index)
    }

    pub(crate) fn constraints_met(
        input_types: &[Self],
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        for input_type in input_types {
            if input_type.keyboard_inputs.is_empty() {
                return true;
            }
            if input_type
                .keyboard_inputs
                .iter()
                .any(|input| input.matches(key, modifiers, is_pressed, is_repeat))
            {
                return true;
            }
        }
        false
    }
}

pub(super) fn key_phase_matches(key_phase: u32, is_pressed: bool, is_repeat: bool) -> bool {
    let mask = key_phase & KEY_PHASE_ALL;
    if mask == 0 {
        return false;
    }
    if is_pressed && is_repeat {
        return mask & KEY_PHASE_REPEAT != 0;
    }
    if is_pressed {
        return mask & KEY_PHASE_DOWN != 0;
    }
    mask & KEY_PHASE_UP != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyboard_input(
        global_id: u32,
        key_type: u32,
        key_phase: u32,
        modifiers: u32,
    ) -> RuntimeKeyboardInput {
        RuntimeKeyboardInput {
            global_id,
            key_type,
            key_phase,
            modifiers,
        }
    }

    #[test]
    fn key_phase_mask_matches_cpp_branch_order() {
        assert!(!key_phase_matches(0, true, false));
        assert!(key_phase_matches(KEY_PHASE_DOWN, true, false));
        assert!(!key_phase_matches(KEY_PHASE_DOWN, true, true));
        assert!(key_phase_matches(KEY_PHASE_REPEAT, true, true));
        assert!(key_phase_matches(KEY_PHASE_UP, false, true));
        assert!(key_phase_matches(KEY_PHASE_ALL, false, false));
        assert!(!key_phase_matches(8, false, false));
    }

    #[test]
    fn keyboard_constraint_uses_wildcard_key_and_exact_modifiers() {
        let wildcard = keyboard_input(1, u32::MAX, KEY_PHASE_DOWN, 2);
        assert!(wildcard.matches(65, 2, true, false));
        assert!(wildcard.matches(66, 2, true, false));
        assert!(!wildcard.matches(65, 0, true, false));

        let exact = keyboard_input(2, 65, KEY_PHASE_UP, 0);
        assert!(exact.matches(65, 0, false, false));
        assert!(!exact.matches(66, 0, false, false));
        assert!(!exact.matches(65, 0, true, false));
    }

    #[test]
    fn keyboard_listener_constraints_scan_authored_types_to_first_match() {
        let nonmatching = RuntimeListenerInputTypeKeyboard {
            global_id: 10,
            keyboard_inputs: vec![keyboard_input(11, 65, KEY_PHASE_DOWN, 0)],
        };
        let matching = RuntimeListenerInputTypeKeyboard {
            global_id: 12,
            keyboard_inputs: vec![keyboard_input(13, 66, KEY_PHASE_UP, 1)],
        };
        assert!(RuntimeListenerInputTypeKeyboard::constraints_met(
            &[nonmatching.clone(), matching],
            66,
            1,
            false,
            false,
        ));
        assert!(!RuntimeListenerInputTypeKeyboard::constraints_met(
            &[nonmatching],
            66,
            1,
            false,
            false,
        ));

        let catch_all = RuntimeListenerInputTypeKeyboard {
            global_id: 14,
            keyboard_inputs: Vec::new(),
        };
        assert!(RuntimeListenerInputTypeKeyboard::constraints_met(
            &[catch_all],
            999,
            7,
            true,
            true,
        ));
        assert!(!RuntimeListenerInputTypeKeyboard::constraints_met(
            &[],
            999,
            7,
            true,
            true,
        ));
    }

    #[test]
    fn keyboard_input_owner_preserves_source_identity_and_order() {
        let owner = RuntimeListenerInputTypeKeyboard {
            global_id: 20,
            keyboard_inputs: vec![
                keyboard_input(21, 65, KEY_PHASE_DOWN, 0),
                keyboard_input(22, 66, KEY_PHASE_UP, 0),
            ],
        };
        assert_eq!(owner.global_id, 20);
        assert_eq!(owner.keyboard_input_count(), 2);
        assert_eq!(
            owner.keyboard_input(0).map(|input| input.global_id),
            Some(21)
        );
        assert_eq!(
            owner.keyboard_input(1).map(|input| input.global_id),
            Some(22)
        );
    }
}
