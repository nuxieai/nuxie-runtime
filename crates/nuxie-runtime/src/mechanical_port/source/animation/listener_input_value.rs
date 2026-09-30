//! include/rive/animation/listener_input_value.hpp at abf676e7.
//! Select an invocation value to write into the action's bindable property.
//! Its type must match that bindable; the to-source converter handles any
//! different view-model property type. Missing values skip the action.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListenerInputValue {
    None = 0,
    PointerX = 1,
    PointerY = 2,
    PointerDeltaX = 3,
    PointerDeltaY = 4,
    KeyPressed = 5,
    Text = 6,
    Focused = 7,
    GamepadButtonPressed = 8,
    GamepadButtonValue = 9,
    GamepadAxis = 10,
    GamepadChangedValue = 11,
}
impl ListenerInputValue {
    pub fn from_value(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::PointerX,
            2 => Self::PointerY,
            3 => Self::PointerDeltaX,
            4 => Self::PointerDeltaY,
            5 => Self::KeyPressed,
            6 => Self::Text,
            7 => Self::Focused,
            8 => Self::GamepadButtonPressed,
            9 => Self::GamepadButtonValue,
            10 => Self::GamepadAxis,
            11 => Self::GamepadChangedValue,
            _ => return Option::None,
        })
    }
}
