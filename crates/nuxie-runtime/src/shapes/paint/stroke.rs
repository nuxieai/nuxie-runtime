use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Stroke", "thickness") != Some(property_key) {
        return None;
    }
    Some(artboard.add_dirt(local_id, ComponentDirt::PAINT, false))
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["cap", "join"]
        .into_iter()
        .any(|name| property_key_for_name("Stroke", name) == Some(property_key))
    {
        return None;
    }
    Some(artboard.add_dirt(local_id, ComponentDirt::PAINT, false))
}
