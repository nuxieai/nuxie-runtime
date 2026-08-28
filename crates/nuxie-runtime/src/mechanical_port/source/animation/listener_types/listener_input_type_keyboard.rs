use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::animation::listener_types::listener_input_type_keyboard_base::ListenerInputTypeKeyboardBase,
};
pub trait KeyboardConstraintListener {
    fn keyboard_input_types(&self) -> Vec<CoreHandle>;
}
#[derive(Default)]
pub struct ListenerInputTypeKeyboard {
    pub base: ListenerInputTypeKeyboardBase,
    keyboard_inputs: Vec<CoreHandle>,
}
impl ListenerInputTypeKeyboard {
    pub fn keyboard_input_count(&self) -> usize {
        self.keyboard_inputs.len()
    }
    pub fn keyboard_input(&self, index: usize) -> Option<CoreHandle> {
        self.keyboard_inputs.get(index).cloned()
    }
    pub fn add_keyboard_input(&mut self, input: CoreHandle) {
        if !self.keyboard_inputs.contains(&input) {
            self.keyboard_inputs.push(input);
        }
    }
    pub fn key_phase_matches(key_phase: u32, pressed: bool, repeat: bool) -> bool {
        let mask = key_phase & 7;
        if mask == 0 {
            return false;
        }
        if pressed && repeat {
            mask & 4 != 0
        } else if pressed {
            mask & 1 != 0
        } else {
            mask & 2 != 0
        }
    }
    pub fn keyboard_input_matches(
        input: &CoreHandle,
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        input
            .with_downcast::<crate::mechanical_port::source::inputs::keyboard_input::KeyboardInput, _>(|input| {
                (input.key_type() == u32::MAX || input.key_type() == key)
                    && input.modifiers() == modifiers
                    && Self::key_phase_matches(input.key_phase(), pressed, repeat)
            })
            .unwrap_or(false)
    }
    pub fn keyboard_listener_constraints_met(
        listener: Option<&dyn KeyboardConstraintListener>,
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        for kind in listener.keyboard_input_types() {
            let Some(matched) = kind.with_downcast::<ListenerInputTypeKeyboard, _>(|kind| {
                if kind.keyboard_input_count() == 0 {
                    return true;
                }
                kind.keyboard_inputs.iter().any(|input| {
                    Self::keyboard_input_matches(input, key, modifiers, pressed, repeat)
                })
            }) else {
                continue;
            };
            if matched {
                return true;
            }
        }
        false
    }
}
