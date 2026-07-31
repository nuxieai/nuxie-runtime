use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["strength", "offsetX", "offsetY"]
        .into_iter()
        .any(|name| property_key_for_name("Feather", name) == Some(property_key))
    {
        return None;
    }
    let inner = property_key_for_name("Feather", "inner")
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(false);
    let dirt = if inner {
        ComponentDirt::PAINT | ComponentDirt::WORLD_TRANSFORM
    } else {
        ComponentDirt::PAINT
    };
    Some(artboard.add_dirt(local_id, dirt, false))
}

// Pinned Feather overrides no generated callback for `inner` or `spaceValue`.
pub(crate) fn bool_property_changed(
    _artboard: &mut ArtboardInstance,
    _local_id: usize,
    _property_key: u16,
) -> Option<bool> {
    // The generated setter reaches an inherited empty callback. Treat it as
    // handled so no generic cache refresh is substituted for that no-op.
    Some(false)
}

pub(crate) fn uint_property_changed(
    _artboard: &mut ArtboardInstance,
    _local_id: usize,
    _property_key: u16,
) -> Option<bool> {
    Some(false)
}
