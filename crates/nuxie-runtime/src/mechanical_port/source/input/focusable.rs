use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, core::CoreHandle,
    semantic::semantic_snapshot::Bounds,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct KeyModifiers(pub u8);
impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1);
    pub const CTRL: Self = Self(2);
    pub const ALT: Self = Self(4);
    pub const META: Self = Self(8);

    pub const fn from_raw(value: u32) -> Self {
        Self(value as u8)
    }
    pub const fn bits(self) -> u32 {
        self.0 as u32
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
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
enum KnownKey {
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

/// Exact `uint16_t` carrier for the C++ `Key` ABI.
///
/// C++ callers can cast values outside the currently named key set. Keeping
/// the carrier open avoids constructing an invalid Rust enum discriminant at
/// an FFI or embedder boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(u16);

impl Key {
    pub const SPACE: Self = Self(KnownKey::Space as u16);
    pub const APOSTROPHE: Self = Self(KnownKey::Apostrophe as u16);
    pub const COMMA: Self = Self(KnownKey::Comma as u16);
    pub const MINUS: Self = Self(KnownKey::Minus as u16);
    pub const PERIOD: Self = Self(KnownKey::Period as u16);
    pub const SLASH: Self = Self(KnownKey::Slash as u16);
    pub const KEY_0: Self = Self(KnownKey::Key0 as u16);
    pub const KEY_1: Self = Self(KnownKey::Key1 as u16);
    pub const KEY_2: Self = Self(KnownKey::Key2 as u16);
    pub const KEY_3: Self = Self(KnownKey::Key3 as u16);
    pub const KEY_4: Self = Self(KnownKey::Key4 as u16);
    pub const KEY_5: Self = Self(KnownKey::Key5 as u16);
    pub const KEY_6: Self = Self(KnownKey::Key6 as u16);
    pub const KEY_7: Self = Self(KnownKey::Key7 as u16);
    pub const KEY_8: Self = Self(KnownKey::Key8 as u16);
    pub const KEY_9: Self = Self(KnownKey::Key9 as u16);
    pub const SEMICOLON: Self = Self(KnownKey::Semicolon as u16);
    pub const EQUAL: Self = Self(KnownKey::Equal as u16);
    pub const A: Self = Self(KnownKey::A as u16);
    pub const B: Self = Self(KnownKey::B as u16);
    pub const C: Self = Self(KnownKey::C as u16);
    pub const D: Self = Self(KnownKey::D as u16);
    pub const E: Self = Self(KnownKey::E as u16);
    pub const F: Self = Self(KnownKey::F as u16);
    pub const G: Self = Self(KnownKey::G as u16);
    pub const H: Self = Self(KnownKey::H as u16);
    pub const I: Self = Self(KnownKey::I as u16);
    pub const J: Self = Self(KnownKey::J as u16);
    pub const K: Self = Self(KnownKey::K as u16);
    pub const L: Self = Self(KnownKey::L as u16);
    pub const M: Self = Self(KnownKey::M as u16);
    pub const N: Self = Self(KnownKey::N as u16);
    pub const O: Self = Self(KnownKey::O as u16);
    pub const P: Self = Self(KnownKey::P as u16);
    pub const Q: Self = Self(KnownKey::Q as u16);
    pub const R: Self = Self(KnownKey::R as u16);
    pub const S: Self = Self(KnownKey::S as u16);
    pub const T: Self = Self(KnownKey::T as u16);
    pub const U: Self = Self(KnownKey::U as u16);
    pub const V: Self = Self(KnownKey::V as u16);
    pub const W: Self = Self(KnownKey::W as u16);
    pub const X: Self = Self(KnownKey::X as u16);
    pub const Y: Self = Self(KnownKey::Y as u16);
    pub const Z: Self = Self(KnownKey::Z as u16);
    pub const LEFT_BRACKET: Self = Self(KnownKey::LeftBracket as u16);
    pub const BACKSLASH: Self = Self(KnownKey::Backslash as u16);
    pub const RIGHT_BRACKET: Self = Self(KnownKey::RightBracket as u16);
    pub const GRAVE_ACCENT: Self = Self(KnownKey::GraveAccent as u16);
    pub const WORLD_1: Self = Self(KnownKey::World1 as u16);
    pub const WORLD_2: Self = Self(KnownKey::World2 as u16);
    pub const ESCAPE: Self = Self(KnownKey::Escape as u16);
    pub const ENTER: Self = Self(KnownKey::Enter as u16);
    pub const TAB: Self = Self(KnownKey::Tab as u16);
    pub const BACKSPACE: Self = Self(KnownKey::Backspace as u16);
    pub const INSERT: Self = Self(KnownKey::Insert as u16);
    pub const DELETE: Self = Self(KnownKey::DeleteKey as u16);
    pub const RIGHT: Self = Self(KnownKey::Right as u16);
    pub const LEFT: Self = Self(KnownKey::Left as u16);
    pub const DOWN: Self = Self(KnownKey::Down as u16);
    pub const UP: Self = Self(KnownKey::Up as u16);
    pub const PAGE_UP: Self = Self(KnownKey::PageUp as u16);
    pub const PAGE_DOWN: Self = Self(KnownKey::PageDown as u16);
    pub const HOME: Self = Self(KnownKey::Home as u16);
    pub const END: Self = Self(KnownKey::End as u16);
    pub const CAPS_LOCK: Self = Self(KnownKey::CapsLock as u16);
    pub const SCROLL_LOCK: Self = Self(KnownKey::ScrollLock as u16);
    pub const NUM_LOCK: Self = Self(KnownKey::NumLock as u16);
    pub const PRINT_SCREEN: Self = Self(KnownKey::PrintScreen as u16);
    pub const PAUSE: Self = Self(KnownKey::Pause as u16);
    pub const F1: Self = Self(KnownKey::F1 as u16);
    pub const F2: Self = Self(KnownKey::F2 as u16);
    pub const F3: Self = Self(KnownKey::F3 as u16);
    pub const F4: Self = Self(KnownKey::F4 as u16);
    pub const F5: Self = Self(KnownKey::F5 as u16);
    pub const F6: Self = Self(KnownKey::F6 as u16);
    pub const F7: Self = Self(KnownKey::F7 as u16);
    pub const F8: Self = Self(KnownKey::F8 as u16);
    pub const F9: Self = Self(KnownKey::F9 as u16);
    pub const F10: Self = Self(KnownKey::F10 as u16);
    pub const F11: Self = Self(KnownKey::F11 as u16);
    pub const F12: Self = Self(KnownKey::F12 as u16);
    pub const F13: Self = Self(KnownKey::F13 as u16);
    pub const F14: Self = Self(KnownKey::F14 as u16);
    pub const F15: Self = Self(KnownKey::F15 as u16);
    pub const F16: Self = Self(KnownKey::F16 as u16);
    pub const F17: Self = Self(KnownKey::F17 as u16);
    pub const F18: Self = Self(KnownKey::F18 as u16);
    pub const F19: Self = Self(KnownKey::F19 as u16);
    pub const F20: Self = Self(KnownKey::F20 as u16);
    pub const F21: Self = Self(KnownKey::F21 as u16);
    pub const F22: Self = Self(KnownKey::F22 as u16);
    pub const F23: Self = Self(KnownKey::F23 as u16);
    pub const F24: Self = Self(KnownKey::F24 as u16);
    pub const F25: Self = Self(KnownKey::F25 as u16);
    pub const KP_0: Self = Self(KnownKey::Kp0 as u16);
    pub const KP_1: Self = Self(KnownKey::Kp1 as u16);
    pub const KP_2: Self = Self(KnownKey::Kp2 as u16);
    pub const KP_3: Self = Self(KnownKey::Kp3 as u16);
    pub const KP_4: Self = Self(KnownKey::Kp4 as u16);
    pub const KP_5: Self = Self(KnownKey::Kp5 as u16);
    pub const KP_6: Self = Self(KnownKey::Kp6 as u16);
    pub const KP_7: Self = Self(KnownKey::Kp7 as u16);
    pub const KP_8: Self = Self(KnownKey::Kp8 as u16);
    pub const KP_9: Self = Self(KnownKey::Kp9 as u16);
    pub const KP_DECIMAL: Self = Self(KnownKey::KpDecimal as u16);
    pub const KP_DIVIDE: Self = Self(KnownKey::KpDivide as u16);
    pub const KP_MULTIPLY: Self = Self(KnownKey::KpMultiply as u16);
    pub const KP_SUBTRACT: Self = Self(KnownKey::KpSubtract as u16);
    pub const KP_ADD: Self = Self(KnownKey::KpAdd as u16);
    pub const KP_ENTER: Self = Self(KnownKey::KpEnter as u16);
    pub const KP_EQUAL: Self = Self(KnownKey::KpEqual as u16);
    pub const LEFT_SHIFT: Self = Self(KnownKey::LeftShift as u16);
    pub const LEFT_CONTROL: Self = Self(KnownKey::LeftControl as u16);
    pub const LEFT_ALT: Self = Self(KnownKey::LeftAlt as u16);
    pub const LEFT_SUPER: Self = Self(KnownKey::LeftSuper as u16);
    pub const RIGHT_SHIFT: Self = Self(KnownKey::RightShift as u16);
    pub const RIGHT_CONTROL: Self = Self(KnownKey::RightControl as u16);
    pub const RIGHT_ALT: Self = Self(KnownKey::RightAlt as u16);
    pub const RIGHT_SUPER: Self = Self(KnownKey::RightSuper as u16);
    pub const MENU: Self = Self(KnownKey::Menu as u16);

    pub const fn from_raw(value: u32) -> Self {
        Self(value as u16)
    }

    pub const fn raw(self) -> u32 {
        self.0 as u32
    }
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
    fn gamepad_dispatch(
        &mut self,
        _invocation: &ListenerInvocation,
        _out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
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
