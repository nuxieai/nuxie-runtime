use crate::{artboard::ArtboardInstance, properties::property_key_for_name};

/// Pinned `LayoutComponent::markLayoutNodeDirty`: dirt the retained Yoga-like
/// owner, then publish the exact component to its Artboard.
pub(crate) fn mark_layout_node_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance.mark_layout_node_changed(local_id)
}

pub(crate) fn mark_layout_style_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance.add_dirt(
        local_id,
        crate::components::ComponentDirt::LAYOUT_STYLE,
        false,
    )
}

pub(crate) fn mark_position_left_changed(instance: &ArtboardInstance, local_id: usize) {
    if let Some(layout) = instance
        .component(local_id)
        .and_then(|component| component.concrete.layout.as_ref())
    {
        layout.mark_position_left_changed();
    }
}

pub(crate) fn mark_position_top_changed(instance: &ArtboardInstance, local_id: usize) {
    if let Some(layout) = instance
        .component(local_id)
        .and_then(|component| component.concrete.layout.as_ref())
    {
        layout.mark_position_top_changed();
    }
}

pub(crate) fn flex_direction_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    mark_layout_node_dirty(instance, local_id)
        | instance.mark_layout_component_children_dirty(local_id)
}

pub(crate) fn direction_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    if let Some(layout) = instance
        .component(local_id)
        .and_then(|component| component.concrete.layout.as_ref())
    {
        layout.force_update_layout_bounds();
    }
    mark_layout_style_dirty(instance, local_id) | mark_layout_node_dirty(instance, local_id)
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if !matches!(type_name, Some("LayoutComponent" | "NestedArtboardLayout")) {
        return None;
    }
    ["width", "height", "fractionalWidth", "fractionalHeight"]
        .into_iter()
        .any(|name| property_key_for_name("LayoutComponent", name) == Some(property_key))
        .then(|| mark_layout_node_dirty(instance, local_id))
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if !matches!(type_name, Some("LayoutComponent" | "NestedArtboardLayout")) {
        return None;
    }
    (property_key_for_name("LayoutComponent", "styleId") == Some(property_key))
        .then(|| mark_layout_node_dirty(instance, local_id))
}

pub(crate) fn bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if !matches!(type_name, Some("LayoutComponent" | "NestedArtboardLayout")) {
        return None;
    }
    (property_key_for_name("LayoutComponent", "clip") == Some(property_key))
        .then(|| mark_layout_node_dirty(instance, local_id))
}
