use crate::mechanical_port::source::semantic::semantic_snapshot::Bounds;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct KeyModifiers(pub u8);
impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1);
    pub const CTRL: Self = Self(2);
    pub const ALT: Self = Self(4);
    pub const META: Self = Self(8);
}
impl core::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl core::ops::BitAnd for KeyModifiers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Key {
    Space = 32,
    Apostrophe = 39,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    Key0 = 48,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon = 59,
    Equal = 61,
    A = 65,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket = 91,
    Backslash,
    RightBracket,
    GraveAccent = 96,
    World1 = 161,
    World2,
    Escape = 256,
    Enter,
    Tab,
    Backspace,
    Insert,
    DeleteKey,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock = 280,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1 = 290,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0 = 320,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift = 340,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
}
pub trait Focusable {
    fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool;
    fn text_input(&mut self, text: &str) -> bool;
    fn gamepad_dispatch(&mut self, _invocation: &dyn core::any::Any) -> bool {
        false
    }
    fn focused(&mut self);
    fn blurred(&mut self);
    fn world_position(&self) -> Option<(f32, f32)> {
        None
    }
    fn world_bounds(&self) -> Option<Bounds> {
        None
    }
    fn is_eligible_for_focus_traversal(&self) -> bool {
        true
    }
    fn accepts_keyboard_input(&self) -> bool {
        false
    }
}
