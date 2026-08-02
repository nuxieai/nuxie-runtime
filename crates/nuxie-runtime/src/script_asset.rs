use crate::scripting::{RuntimeScriptInstanceHandle, ScriptInstance};
use std::cell::RefMut;

/// Pinned C++ `OptionalScriptedMethods` serialized bitfield.
///
/// Files predating property 1022 decode to the generated all-bits default.
/// Runtime registration consults these authored bits; it does not rediscover
/// listener membership from the live Lua table (`script_asset.cpp:145-159`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeScriptImplementedMethods(u32);

impl RuntimeScriptImplementedMethods {
    pub(crate) const METHOD_MASK: u32 = (1 << 21) - 1;
    pub(crate) const LEGACY_ALL: Self = Self(Self::METHOD_MASK);

    pub(crate) const ADVANCE: u32 = 1 << 0;
    pub(crate) const MEASURE: u32 = 1 << 2;
    pub(crate) const POINTER_DOWN: u32 = 1 << 3;
    pub(crate) const POINTER_MOVE: u32 = 1 << 4;
    pub(crate) const POINTER_UP: u32 = 1 << 5;
    pub(crate) const POINTER_EXIT: u32 = 1 << 6;
    pub(crate) const POINTER_CANCEL: u32 = 1 << 7;
    pub(crate) const INIT: u32 = 1 << 9;
    pub(crate) const DATA_CONVERT: u32 = 1 << 10;
    pub(crate) const DATA_REVERSE_CONVERT: u32 = 1 << 11;
    pub(crate) const RESIZE: u32 = 1 << 12;
    pub(crate) const LISTENER_PERFORM: u32 = 1 << 13;
    pub(crate) const LISTENER_PERFORM_ACTION: u32 = 1 << 14;
    pub(crate) const KEYBOARD: u32 = 1 << 16;
    pub(crate) const TEXT: u32 = 1 << 17;
    pub(crate) const GAMEPAD_CONNECT: u32 = 1 << 18;
    pub(crate) const GAMEPAD_DISCONNECT: u32 = 1 << 19;
    pub(crate) const GAMEPAD_EVENT: u32 = 1 << 20;

    pub(crate) fn from_serialized(value: u32) -> Self {
        Self(value & Self::METHOD_MASK)
    }

    pub(crate) fn wants_keyboard(self) -> bool {
        self.0 & Self::KEYBOARD != 0
    }

    pub(crate) fn advances(self) -> bool {
        self.0 & Self::ADVANCE != 0
    }

    pub(crate) fn measures(self) -> bool {
        self.0 & Self::MEASURE != 0
    }

    pub(crate) fn resizes(self) -> bool {
        self.0 & Self::RESIZE != 0
    }

    pub(crate) fn listens_to_pointer_events(self) -> bool {
        self.0
            & (Self::POINTER_DOWN
                | Self::POINTER_MOVE
                | Self::POINTER_UP
                | Self::POINTER_EXIT
                | Self::POINTER_CANCEL
                | Self::GAMEPAD_CONNECT
                | Self::GAMEPAD_DISCONNECT
                | Self::GAMEPAD_EVENT)
            != 0
    }

    pub(crate) fn wants_pointer_down(self) -> bool {
        self.0 & Self::POINTER_DOWN != 0
    }

    pub(crate) fn wants_pointer_move(self) -> bool {
        self.0 & Self::POINTER_MOVE != 0
    }

    pub(crate) fn wants_pointer_up(self) -> bool {
        self.0 & Self::POINTER_UP != 0
    }

    pub(crate) fn wants_pointer_exit(self) -> bool {
        self.0 & Self::POINTER_EXIT != 0
    }

    pub(crate) fn inits(self) -> bool {
        self.0 & Self::INIT != 0
    }

    pub(crate) fn data_converts(self) -> bool {
        self.0 & Self::DATA_CONVERT != 0
    }

    pub(crate) fn data_reverse_converts(self) -> bool {
        self.0 & Self::DATA_REVERSE_CONVERT != 0
    }

    pub(crate) fn wants_text(self) -> bool {
        self.0 & Self::TEXT != 0
    }

    pub(crate) fn wants_gamepad_connect(self) -> bool {
        self.0 & Self::GAMEPAD_CONNECT != 0
    }

    pub(crate) fn wants_gamepad_disconnect(self) -> bool {
        self.0 & Self::GAMEPAD_DISCONNECT != 0
    }

    pub(crate) fn wants_gamepad_event(self) -> bool {
        self.0 & Self::GAMEPAD_EVENT != 0
    }
}

/// One concrete scripted-object occurrence: VM table plus the authored method
/// mask copied from its ScriptAsset.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeScriptedObjectOccurrence {
    instance: RuntimeScriptInstanceHandle,
    implemented_methods: RuntimeScriptImplementedMethods,
}

impl RuntimeScriptedObjectOccurrence {
    pub(crate) fn new(
        instance: Box<dyn ScriptInstance>,
        serialized_implemented_methods: u32,
    ) -> Self {
        Self {
            instance: RuntimeScriptInstanceHandle::new(instance),
            implemented_methods: RuntimeScriptImplementedMethods::from_serialized(
                serialized_implemented_methods,
            ),
        }
    }

    pub(crate) fn instance(&self) -> RuntimeScriptInstanceHandle {
        self.instance.clone()
    }

    pub(crate) fn implemented_methods(&self) -> RuntimeScriptImplementedMethods {
        self.implemented_methods
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Box<dyn ScriptInstance>> {
        self.instance.borrow_mut()
    }
}

/// Pinned `ScriptAsset::inits()` for facade-owned scripted occurrences.
///
/// Keep the bit interpretation on the ScriptAsset owner so every mount path
/// shares the generated all-bits legacy default and the same masked authored
/// field (`script_asset.cpp:145-161`).
#[doc(hidden)]
pub fn scripted_object_inits(serialized_implemented_methods: u32) -> bool {
    RuntimeScriptImplementedMethods::from_serialized(serialized_implemented_methods).inits()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_script_method_bits_are_masked_and_independently_gated_like_cpp() {
        let legacy = RuntimeScriptImplementedMethods::from_serialized(u32::MAX);
        assert_eq!(legacy, RuntimeScriptImplementedMethods::LEGACY_ALL);
        assert!(legacy.advances());
        assert!(legacy.measures());
        assert!(legacy.resizes());
        assert!(legacy.listens_to_pointer_events());
        assert!(legacy.wants_pointer_down());
        assert!(legacy.wants_pointer_move());
        assert!(legacy.wants_pointer_up());
        assert!(legacy.wants_pointer_exit());
        assert!(legacy.inits());
        assert!(legacy.data_converts());
        assert!(legacy.data_reverse_converts());
        assert!(legacy.wants_keyboard());
        assert!(legacy.wants_text());
        assert!(legacy.wants_gamepad_connect());
        assert!(legacy.wants_gamepad_disconnect());
        assert!(legacy.wants_gamepad_event());

        let convert = RuntimeScriptImplementedMethods::from_serialized(
            RuntimeScriptImplementedMethods::DATA_CONVERT,
        );
        assert!(convert.data_converts());
        assert!(!convert.data_reverse_converts());
        assert!(!convert.advances());
        assert!(!convert.measures());
        assert!(!convert.resizes());
        assert!(!convert.inits());

        let reverse = RuntimeScriptImplementedMethods::from_serialized(
            RuntimeScriptImplementedMethods::DATA_REVERSE_CONVERT,
        );
        assert!(!reverse.data_converts());
        assert!(reverse.data_reverse_converts());

        let none = RuntimeScriptImplementedMethods::from_serialized(
            !RuntimeScriptImplementedMethods::METHOD_MASK,
        );
        assert_eq!(none, RuntimeScriptImplementedMethods(0));
    }
}
