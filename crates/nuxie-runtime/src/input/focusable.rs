//! Typed retained `Focusable*` relation used by the Rust focus tree.
//!
//! This is the Rust ownership adapter for the pointer retained by pinned C++
//! `FocusNode`; concrete `Focusable` callbacks remain owned by their live
//! `FocusData`, `TextInput`, and `NestedArtboard` dispatch paths.

use std::ops::{BitAnd, BitOr};

/// Exact `uint16_t` carrier for pinned C++ `Key`.
///
/// A newtype, rather than a Rust enum, preserves C++ casts containing an
/// unknown discriminant. The public embedder API remains raw `u32`; it
/// truncates once at that ABI boundary before entering the focus pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(u16);

impl Key {
    pub const SPACE: Self = Self(32);
    pub const APOSTROPHE: Self = Self(39);
    pub const COMMA: Self = Self(44);
    pub const MINUS: Self = Self(45);
    pub const PERIOD: Self = Self(46);
    pub const SLASH: Self = Self(47);
    pub const KEY_0: Self = Self(48);
    pub const KEY_1: Self = Self(49);
    pub const KEY_2: Self = Self(50);
    pub const KEY_3: Self = Self(51);
    pub const KEY_4: Self = Self(52);
    pub const KEY_5: Self = Self(53);
    pub const KEY_6: Self = Self(54);
    pub const KEY_7: Self = Self(55);
    pub const KEY_8: Self = Self(56);
    pub const KEY_9: Self = Self(57);
    pub const SEMICOLON: Self = Self(59);
    pub const EQUAL: Self = Self(61);
    pub const A: Self = Self(65);
    pub const B: Self = Self(66);
    pub const C: Self = Self(67);
    pub const D: Self = Self(68);
    pub const E: Self = Self(69);
    pub const F: Self = Self(70);
    pub const G: Self = Self(71);
    pub const H: Self = Self(72);
    pub const I: Self = Self(73);
    pub const J: Self = Self(74);
    pub const K: Self = Self(75);
    pub const L: Self = Self(76);
    pub const M: Self = Self(77);
    pub const N: Self = Self(78);
    pub const O: Self = Self(79);
    pub const P: Self = Self(80);
    pub const Q: Self = Self(81);
    pub const R: Self = Self(82);
    pub const S: Self = Self(83);
    pub const T: Self = Self(84);
    pub const U: Self = Self(85);
    pub const V: Self = Self(86);
    pub const W: Self = Self(87);
    pub const X: Self = Self(88);
    pub const Y: Self = Self(89);
    pub const Z: Self = Self(90);
    pub const LEFT_BRACKET: Self = Self(91);
    pub const BACKSLASH: Self = Self(92);
    pub const RIGHT_BRACKET: Self = Self(93);
    pub const GRAVE_ACCENT: Self = Self(96);
    pub const WORLD_1: Self = Self(161);
    pub const WORLD_2: Self = Self(162);
    pub const ESCAPE: Self = Self(256);
    pub const ENTER: Self = Self(257);
    pub const TAB: Self = Self(258);
    pub const BACKSPACE: Self = Self(259);
    pub const INSERT: Self = Self(260);
    pub const DELETE: Self = Self(261);
    pub const RIGHT: Self = Self(262);
    pub const LEFT: Self = Self(263);
    pub const DOWN: Self = Self(264);
    pub const UP: Self = Self(265);
    pub const PAGE_UP: Self = Self(266);
    pub const PAGE_DOWN: Self = Self(267);
    pub const HOME: Self = Self(268);
    pub const END: Self = Self(269);
    pub const CAPS_LOCK: Self = Self(280);
    pub const SCROLL_LOCK: Self = Self(281);
    pub const NUM_LOCK: Self = Self(282);
    pub const PRINT_SCREEN: Self = Self(283);
    pub const PAUSE: Self = Self(284);
    pub const F1: Self = Self(290);
    pub const F2: Self = Self(291);
    pub const F3: Self = Self(292);
    pub const F4: Self = Self(293);
    pub const F5: Self = Self(294);
    pub const F6: Self = Self(295);
    pub const F7: Self = Self(296);
    pub const F8: Self = Self(297);
    pub const F9: Self = Self(298);
    pub const F10: Self = Self(299);
    pub const F11: Self = Self(300);
    pub const F12: Self = Self(301);
    pub const F13: Self = Self(302);
    pub const F14: Self = Self(303);
    pub const F15: Self = Self(304);
    pub const F16: Self = Self(305);
    pub const F17: Self = Self(306);
    pub const F18: Self = Self(307);
    pub const F19: Self = Self(308);
    pub const F20: Self = Self(309);
    pub const F21: Self = Self(310);
    pub const F22: Self = Self(311);
    pub const F23: Self = Self(312);
    pub const F24: Self = Self(313);
    pub const F25: Self = Self(314);
    pub const KP_0: Self = Self(320);
    pub const KP_1: Self = Self(321);
    pub const KP_2: Self = Self(322);
    pub const KP_3: Self = Self(323);
    pub const KP_4: Self = Self(324);
    pub const KP_5: Self = Self(325);
    pub const KP_6: Self = Self(326);
    pub const KP_7: Self = Self(327);
    pub const KP_8: Self = Self(328);
    pub const KP_9: Self = Self(329);
    pub const KP_DECIMAL: Self = Self(330);
    pub const KP_DIVIDE: Self = Self(331);
    pub const KP_MULTIPLY: Self = Self(332);
    pub const KP_SUBTRACT: Self = Self(333);
    pub const KP_ADD: Self = Self(334);
    pub const KP_ENTER: Self = Self(335);
    pub const KP_EQUAL: Self = Self(336);
    pub const LEFT_SHIFT: Self = Self(340);
    pub const LEFT_CONTROL: Self = Self(341);
    pub const LEFT_ALT: Self = Self(342);
    pub const LEFT_SUPER: Self = Self(343);
    pub const RIGHT_SHIFT: Self = Self(344);
    pub const RIGHT_CONTROL: Self = Self(345);
    pub const RIGHT_ALT: Self = Self(346);
    pub const RIGHT_SUPER: Self = Self(347);
    pub const MENU: Self = Self(348);

    pub const fn from_raw(value: u32) -> Self {
        Self(value as u16)
    }

    pub const fn raw(self) -> u32 {
        self.0 as u32
    }
}

/// Exact `uint8_t` bit carrier for pinned C++ `KeyModifiers`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const META: Self = Self(1 << 3);

    pub const fn from_raw(value: u32) -> Self {
        Self(value as u8)
    }

    pub const fn bits(self) -> u32 {
        self.0 as u32
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for KeyModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for KeyModifiers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Result of pinned `Focusable::from(Core*)` under arena-owned Core identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeFocusableCoreKind {
    TextInput,
    NestedArtboard,
}

impl RuntimeFocusableCoreKind {
    pub(crate) fn from_core_type_name(type_name: Option<&str>) -> Option<Self> {
        let definition = nuxie_schema::definition_by_name(type_name?)?;
        if definition.is_a("TextInput") {
            Some(Self::TextInput)
        } else if definition.is_a("NestedArtboard") {
            Some(Self::NestedArtboard)
        } else {
            None
        }
    }
}

/// Typed owner-local identity for the `Focusable*` retained by pinned C++
/// `FocusNode`. The owner and exact `FocusData` occurrence replace the raw
/// pointer while preserving one live relationship across tree reparenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RuntimeFocusable {
    pub(crate) owner_identity: u64,
    pub(crate) target_local: usize,
    pub(crate) focus_data_local: usize,
    pub(crate) accepts_keyboard_input: bool,
}

impl RuntimeFocusable {
    pub(crate) fn new(owner_identity: u64, target_local: usize, focus_data_local: usize) -> Self {
        Self {
            owner_identity,
            target_local,
            focus_data_local,
            accepts_keyboard_input: false,
        }
    }
}
