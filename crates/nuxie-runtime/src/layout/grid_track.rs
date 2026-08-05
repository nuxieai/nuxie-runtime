use crate::{artboard::ArtboardInstance, layout_component, properties::property_key_for_name};

/// Pinned `GridTrack::markLayoutDirty` (`grid_track.cpp:10-18`): a track dirties
/// the layout node of its immediate parent, and only when that parent is a
/// LayoutComponent. C++ does not walk further up here -- `markDirtyAndPropagate`
/// on the parent's own retained node owns the upward half.
pub(crate) fn mark_layout_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    let Some(parent_local) = instance.component_parent_local(local_id) else {
        return false;
    };
    if !instance
        .component(parent_local)
        .is_some_and(|component| component.concrete.layout.is_some())
    {
        return false;
    }
    layout_component::mark_layout_node_dirty(instance, parent_local)
}

/// `collectionChanged`, `trackTypeChanged`, `trackMaxTypeChanged`
/// (`grid_track.cpp:20,21,23`).
pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("GridTrack") {
        return None;
    }
    ["collection", "trackType", "trackMaxType"]
        .into_iter()
        .any(|name| property_key_for_name("GridTrack", name) == Some(property_key))
        .then(|| mark_layout_dirty(instance, local_id))
}

/// `trackValueChanged`, `trackMaxValueChanged` (`grid_track.cpp:22,24`).
pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("GridTrack") {
        return None;
    }
    ["trackValue", "trackMaxValue"]
        .into_iter()
        .any(|name| property_key_for_name("GridTrack", name) == Some(property_key))
        .then(|| mark_layout_dirty(instance, local_id))
}
