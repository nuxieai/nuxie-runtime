#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StandardGamepadButton {
    South = 0,
    East = 1,
    West = 2,
    North = 3,
    LeftShoulder = 4,
    RightShoulder = 5,
    LeftTrigger = 6,
    RightTrigger = 7,
    Back = 8,
    Forward = 9,
    LeftStick = 10,
    RightStick = 11,
    DpadUp = 12,
    DpadDown = 13,
    DpadLeft = 14,
    DpadRight = 15,
    Start = 16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StandardGamepadAxis {
    LeftX = 0,
    LeftY = 1,
    RightX = 2,
    RightY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
}
pub fn standard_gamepad_axis_value(axes: &[f32], axis: StandardGamepadAxis) -> f32 {
    axes.get(axis as usize).copied().unwrap_or(0.0)
}
pub fn standard_gamepad_button_index(button: StandardGamepadButton) -> usize {
    button as usize
}
pub fn standard_gamepad_button_pressed(mask: u64, button: StandardGamepadButton) -> bool {
    mask & (1 << standard_gamepad_button_index(button)) != 0
}
