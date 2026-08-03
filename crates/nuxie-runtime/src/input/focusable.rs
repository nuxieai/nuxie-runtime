//! Typed retained Focusable relation port of pinned src/input/focusable.cpp (B6-0240).

/// Typed owner-local identity for the `Focusable*` retained by pinned C++
/// `FocusNode`. The owner and exact `FocusData` occurrence replace the raw
/// pointer while preserving one live relationship across tree reparenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RuntimeFocusable {
    pub(crate) owner_identity: u64,
    pub(crate) target_local: usize,
    pub(crate) focus_data_local: usize,
    pub(crate) accepts_keyboard_input: bool,
    kind: RuntimeFocusableKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RuntimeFocusableKind {
    TextInput,
    NestedArtboard,
}

impl RuntimeFocusable {
    pub(crate) fn new(owner_identity: u64, target_local: usize, focus_data_local: usize) -> Self {
        Self {
            owner_identity,
            target_local,
            focus_data_local,
            accepts_keyboard_input: false,
            kind: RuntimeFocusableKind::TextInput,
        }
    }

    pub(crate) fn from_component_type(
        owner_identity: u64,
        target_local: usize,
        focus_data_local: usize,
        type_name: &str,
    ) -> Option<Self> {
        let kind = match type_name {
            "TextInput" => RuntimeFocusableKind::TextInput,
            "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf" => {
                RuntimeFocusableKind::NestedArtboard
            }
            _ => return None,
        };
        Some(Self {
            owner_identity,
            target_local,
            focus_data_local,
            accepts_keyboard_input: false,
            kind,
        })
    }

    /// Pinned `Focusable::gamepadDispatch` base implementation.
    pub(crate) fn gamepad_dispatch_default(self) -> bool {
        let _ = self.kind;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeFocusable;

    #[test]
    fn upstream_focusable_from_and_default_gamepad_dispatch() {
        assert!(RuntimeFocusable::from_component_type(1, 2, 3, "Shape").is_none());
        let text = RuntimeFocusable::from_component_type(1, 2, 3, "TextInput")
            .expect("TextInput implements Focusable");
        let nested = RuntimeFocusable::from_component_type(1, 2, 3, "NestedArtboard")
            .expect("NestedArtboard implements Focusable");
        assert!(!text.gamepad_dispatch_default());
        assert!(!nested.gamepad_dispatch_default());
    }
}
