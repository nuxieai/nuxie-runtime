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
impl StandardGamepadButton {
    pub const fn from_raw(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::South,
            1 => Self::East,
            2 => Self::West,
            3 => Self::North,
            4 => Self::LeftShoulder,
            5 => Self::RightShoulder,
            6 => Self::LeftTrigger,
            7 => Self::RightTrigger,
            8 => Self::Back,
            9 => Self::Forward,
            10 => Self::LeftStick,
            11 => Self::RightStick,
            12 => Self::DpadUp,
            13 => Self::DpadDown,
            14 => Self::DpadLeft,
            15 => Self::DpadRight,
            16 => Self::Start,
            _ => return None,
        })
    }
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
impl StandardGamepadAxis {
    pub const fn from_raw(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::LeftX,
            1 => Self::LeftY,
            2 => Self::RightX,
            3 => Self::RightY,
            4 => Self::LeftTrigger,
            5 => Self::RightTrigger,
            _ => return None,
        })
    }
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
