use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    core::CoreHandle,
    generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase,
    input::gamepad_snapshot::GamepadInputChangeKind,
    inputs::{
        gamepad_button_phase::GamepadButtonPhaseMask,
        gamepad_input_kind::{GamepadInputKind, GamepadInputMapping},
    },
};
pub trait GamepadConstraintListener {
    fn gamepad_input_types(&self) -> Vec<CoreHandle>;
}
#[derive(Default)]
pub struct ListenerInputTypeGamepad {
    pub base: ListenerInputTypeGamepadBase,
    gamepad_inputs: Vec<CoreHandle>,
}
impl ListenerInputTypeGamepad {
    pub fn gamepad_input_count(&self) -> usize {
        self.gamepad_inputs.len()
    }
    pub fn gamepad_input(&self, index: usize) -> Option<CoreHandle> {
        self.gamepad_inputs.get(index).cloned()
    }
    pub fn add_gamepad_input(&mut self, input: CoreHandle) {
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
    pub fn gamepad_input_matches(input: &CoreHandle, invocation: &ListenerInvocation) -> bool {
        input
            .with_downcast::<crate::mechanical_port::source::inputs::gamepad_input::GamepadInput, _>(|input| match input.kind() {
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
                if input.mapping() == GamepadInputMapping::Standard as u32 {
                    if wants_button {
                        if event.standard_button.map(|button| button as u32)
                            != Some(input.input_index())
                        {
                            return false;
                        }
                    } else if event.standard_axis.map(|axis| axis as u32)
                        != Some(input.input_index())
                    {
                        return false;
                    }
                } else if event.change.index as u32 != input.input_index() {
                    return false;
                }
                if wants_button {
                    Self::gamepad_button_phase_matches(
                        input.button_phase(),
                        event.change.value >= 0.5,
                    )
                } else {
                    true
                }
            }
            _ => false,
        }).unwrap_or(false)
    }
    pub fn gamepad_listener_constraints_met(
        listener: Option<&dyn GamepadConstraintListener>,
        invocation: &ListenerInvocation,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        for kind in listener.gamepad_input_types() {
            let Some(matched) = kind.with_downcast::<ListenerInputTypeGamepad, _>(|kind| {
                if kind.gamepad_input_count() == 0 {
                    return true;
                }
                kind.gamepad_inputs
                    .iter()
                    .any(|input| Self::gamepad_input_matches(input, invocation))
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
