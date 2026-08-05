use crate::{artboard::ArtboardInstance, layout_node_provider, properties::property_key_for_name};

/// Pinned `GridItemPlacement::markOwnerDirty` (`grid_item_placement.cpp:73-79`):
/// dirty the layout node of the provider that owns this placement's parent. C++
/// resolves both shapes through `LayoutNodeProvider::from(parent())` -- a
/// LayoutComponent provides for itself, while Text/Image/Shape provide via their
/// LayoutParticipant child, whose own `markLayoutNodeDirty` forwards to the
/// owning layout (`layout_participant.cpp:457-469`).
///
/// `layout_node_provider::mark_layout_node_dirty` walks up from this object's
/// parent and covers both: it marks the parent itself when the parent is a
/// retained LayoutComponent, and otherwise continues to the owning layout.
pub(crate) fn mark_owner_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    layout_node_provider::mark_layout_node_dirty(instance, local_id)
}

/// `gridColumnChanged`, `gridRowChanged` (`grid_item_placement.cpp:81-82`).
pub(crate) fn int_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("GridItemPlacement") {
        return None;
    }
    ["gridColumn", "gridRow"]
        .into_iter()
        .any(|name| property_key_for_name("GridItemPlacement", name) == Some(property_key))
        .then(|| mark_owner_dirty(instance, local_id))
}

/// `gridColumnSpanChanged`, `gridRowSpanChanged`
/// (`grid_item_placement.cpp:83-84`).
pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("GridItemPlacement") {
        return None;
    }
    ["gridColumnSpan", "gridRowSpan"]
        .into_iter()
        .any(|name| property_key_for_name("GridItemPlacement", name) == Some(property_key))
        .then(|| mark_owner_dirty(instance, local_id))
}
