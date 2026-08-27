use crate::mechanical_port::source::{
    generated::animation::listener_types::listener_input_type_keyboard_base::ListenerInputTypeKeyboardBase,
    inputs::keyboard_input::KeyboardInput,
};
use std::ptr::NonNull;
pub trait KeyboardConstraintListener {
    fn keyboard_input_types(&self) -> Vec<&ListenerInputTypeKeyboard>;
}
#[derive(Default)]
pub struct ListenerInputTypeKeyboard {
    pub base: ListenerInputTypeKeyboardBase,
    keyboard_inputs: Vec<NonNull<KeyboardInput>>,
}
impl ListenerInputTypeKeyboard {
    pub fn keyboard_input_count(&self) -> usize {
        self.keyboard_inputs.len()
    }
    pub fn keyboard_input(&self, index: usize) -> Option<&KeyboardInput> {
        self.keyboard_inputs
            .get(index)
            .map(|v| unsafe { v.as_ref() })
    }
    pub fn add_keyboard_input(&mut self, input: &mut KeyboardInput) {
        let input = NonNull::from(input);
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
        input: &KeyboardInput,
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        (input.base.key_type() == u32::MAX || input.base.key_type() == key)
            && input.base.modifiers() == modifiers
            && Self::key_phase_matches(input.base.key_phase(), pressed, repeat)
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
            if kind.keyboard_input_count() == 0 {
                return true;
            }
            if kind.keyboard_inputs.iter().any(|input| {
                Self::keyboard_input_matches(
                    unsafe { input.as_ref() },
                    key,
                    modifiers,
                    pressed,
                    repeat,
                )
            }) {
                return true;
            }
        }
        false
    }
}
