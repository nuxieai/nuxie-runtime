//! Direct Rust owner for pinned C++ `include/rive/virtualizing_component.hpp`
//! and `src/virtualizing_component.cpp`.
//!
//! C++ `VirtualizingComponent::from` accepts only an
//! `ArtboardComponentList`. Rust addresses imported occurrences by local ID,
//! so the corresponding adapter returns that same ID only when its retained
//! occurrence owns component-list state.

use crate::ArtboardInstance;

pub(crate) fn from(instance: &ArtboardInstance, local_id: usize) -> Option<usize> {
    instance
        .component_list_state(local_id)
        .is_some()
        .then_some(local_id)
}
