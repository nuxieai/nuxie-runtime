//! Direct Rust home for `include/rive/text/text_variation_helper.hpp` and
//! `src/text/text_variation_helper.cpp`.
//!
//! Construction and dependency-edge insertion remain in the Artboard
//! occurrence builder because they are interleaved with every Component's
//! authored-order lifecycle. The helper's concrete update callback lives here.

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};

pub(crate) fn update(instance: &mut ArtboardInstance, text: ComponentHandle, dirt: ComponentDirt) {
    if !dirt.contains(ComponentDirt::TEXT_SHAPE) {
        return;
    }
    if let Some(text_local) = instance.component_local_id(text) {
        // C++ rebuilds the variation-bearing Font on the helper update
        // (`src/text/text_variation_helper.cpp:14-17`,
        // `src/text/text_style.cpp:98-124`). Rust's retained text owner
        // rebuilds lazily from the same live axis values, so invalidate
        // precisely that Text occurrence here.
        instance
            .runtime_drawables
            .mark_text_resource_dirty_for_local(text_local);
    }
}
