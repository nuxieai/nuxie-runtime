//! Direct owner for pinned `src/draw_target.cpp`.

use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, ComponentDirt};

/// Rust counterpart of one retained C++ `DrawTarget`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeDrawTarget {
    pub(crate) local_id: usize,
    /// Resolved once by `DrawTarget::onAddedDirty`. The generated
    /// `drawableId` setter has no changed override and therefore does not
    /// relink this owner after construction. Pinned C++ rejects a missing or
    /// non-Drawable id with `StatusCode::MissingObject`; Rust's graph layer
    /// intentionally preserves unresolved references for diagnostics, so the
    /// runtime retains `None` as an inert target instead of rejecting the
    /// entire Artboard import.
    pub(crate) drawable_index: Option<usize>,
    pub(crate) placement_value: u64,
    pub(crate) first: Option<usize>,
    pub(crate) last: Option<usize>,
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("DrawTarget")
        || property_key_for_name("DrawTarget", "placementValue") != Some(property_key)
    {
        return None;
    }

    let placement_value = artboard
        .uint_property(local_id, property_key)
        .unwrap_or(u64::MAX);
    artboard
        .runtime_drawables
        .set_draw_target_placement(local_id, placement_value);
    Some(artboard.add_dirt(0, ComponentDirt::DRAW_ORDER, false))
}
