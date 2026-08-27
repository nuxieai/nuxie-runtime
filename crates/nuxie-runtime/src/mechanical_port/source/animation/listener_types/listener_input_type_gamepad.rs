use crate::mechanical_port::source::{
    animation::listener_invocation::{ListenerInvocation, ListenerInvocationKind},
    generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase,
    inputs::gamepad_input::GamepadInput,
};
use std::ptr::NonNull;
pub trait GamepadConstraintListener {
    fn gamepad_input_types(&self) -> Vec<&ListenerInputTypeGamepad>;
}
#[derive(Default)]
pub struct ListenerInputTypeGamepad {
    pub base: ListenerInputTypeGamepadBase,
    gamepad_inputs: Vec<NonNull<GamepadInput>>,
}
impl ListenerInputTypeGamepad {
    pub fn gamepad_input_count(&self) -> usize {
        self.gamepad_inputs.len()
    }
    pub fn gamepad_input(&self, index: usize) -> Option<&GamepadInput> {
        self.gamepad_inputs
            .get(index)
            .map(|v| unsafe { v.as_ref() })
    }
    pub fn add_gamepad_input(&mut self, input: &mut GamepadInput) {
        let input = NonNull::from(input);
        if !self.gamepad_inputs.contains(&input) {
            self.gamepad_inputs.push(input);
        }
    }
    pub fn gamepad_button_phase_matches(phase: u32, pressed: bool) -> bool {
        let mask = phase & 3;
        mask != 0
            && if pressed {
                mask & 1 != 0
            } else {
                mask & 2 != 0
            }
    }
    pub fn gamepad_input_matches(input: &GamepadInput, invocation: &ListenerInvocation) -> bool {
        invocation.matches_gamepad_input(input)
    }
    pub fn gamepad_listener_constraints_met(
        listener: Option<&dyn GamepadConstraintListener>,
        invocation: &ListenerInvocation,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        for kind in listener.gamepad_input_types() {
            if kind.gamepad_input_count() == 0 {
                return true;
            }
            if kind
                .gamepad_inputs
                .iter()
                .any(|input| Self::gamepad_input_matches(unsafe { input.as_ref() }, invocation))
            {
                return true;
            }
        }
        false
    }
}
