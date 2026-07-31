//! Structural list cache owned by C++ `ContextValueList`.

pub(crate) fn item_count_changed(previous: usize, next: usize) -> bool {
    previous != next
}
