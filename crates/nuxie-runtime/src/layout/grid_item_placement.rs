use crate::{artboard::ArtboardInstance, layout_node_provider, properties::property_key_for_name};

/// Pinned `GridItemPlacement::markOwnerDirty` (`grid_item_placement.cpp:73-79`):
/// dirty the layout node of the provider that owns this placement's parent. C++
/// resolves both shapes through `LayoutNodeProvider::from(parent())` -- a
/// LayoutComponent provides for itself, while Text/Image/Shape provide via their
/// LayoutParticipant child, whose own `markLayoutNodeDirty` forwards to the
/// owning layout (`layout_participant.cpp:457-469`).
///
/// Taffy's retained-layout invalidation is the approved adapter for the
/// provider's virtual `markLayoutNodeDirty`, but provider eligibility remains
/// literal: resolve it from the placement's immediate authored parent first.
/// In particular, a bare Text/Image/Shape without a LayoutParticipant must not
/// fall through to an enclosing LayoutComponent.
pub(crate) fn mark_owner_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    let Some(parent) = instance
        .component_handle(local_id)
        .and_then(|placement| instance.component_parent_handle(placement))
    else {
        return false;
    };
    if layout_node_provider::from(&instance.objects, Some(parent)).is_none() {
        return false;
    }
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
