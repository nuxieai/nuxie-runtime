//! Typed retained `Focusable*` relation used by the Rust focus tree.
//!
//! This is the Rust ownership adapter for the pointer retained by pinned C++
//! `FocusNode`; concrete `Focusable` callbacks remain owned by their live
//! `FocusData`, `TextInput`, and `NestedArtboard` dispatch paths.

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
