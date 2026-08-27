use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase,
    input::gamepad_snapshot::GamepadInputChangeKind,
    inputs::{
        gamepad_button_phase::GamepadButtonPhaseMask,
        gamepad_input::GamepadInput,
        gamepad_input_kind::{GamepadInputKind, GamepadInputMapping},
    },
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
        let mask = phase & GamepadButtonPhaseMask::ALL;
        mask != 0
            && if pressed {
                mask & GamepadButtonPhaseMask::DOWN != 0
            } else {
                mask & GamepadButtonPhaseMask::UP != 0
            }
    }
    pub fn gamepad_input_matches(input: &GamepadInput, invocation: &ListenerInvocation) -> bool {
        match input.base.kind() {
            value if value == GamepadInputKind::Connected as u32 => {
                invocation.as_gamepad_connected().is_some()
            }
            value if value == GamepadInputKind::Disconnected as u32 => {
                invocation.as_gamepad_disconnected().is_some()
            }
            value
                if value == GamepadInputKind::Button as u32
                    || value == GamepadInputKind::Axis as u32 =>
            {
                let Some(event) = invocation.as_gamepad_event() else {
                    return false;
                };
                let wants_button = value == GamepadInputKind::Button as u32;
                let expected = if wants_button {
                    GamepadInputChangeKind::Button
                } else {
                    GamepadInputChangeKind::Axis
                };
                if event.change.kind != expected {
                    return false;
                }
                if input.base.mapping() == GamepadInputMapping::Standard as u32 {
                    if wants_button {
                        if !event.has_standard_button_intent
                            || event.standard_button as u32 != input.base.input_index()
                        {
                            return false;
                        }
                    } else if !event.has_standard_axis_intent
                        || event.standard_axis as u32 != input.base.input_index()
                    {
                        return false;
                    }
                } else if event.change.index as u32 != input.base.input_index() {
                    return false;
                }
                if wants_button {
                    Self::gamepad_button_phase_matches(
                        input.base.button_phase(),
                        event.change.value >= 0.5,
                    )
                } else {
                    true
                }
            }
            _ => false,
        }
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
