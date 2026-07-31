use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if ["startX", "startY", "endX", "endY"]
        .into_iter()
        .any(|name| property_key_for_name("LinearGradient", name) == Some(property_key))
    {
        return Some(artboard.add_dirt(local_id, ComponentDirt::TRANSFORM, false));
    }
    if property_key_for_name("LinearGradient", "opacity") != Some(property_key) {
        return None;
    }
    Some(artboard.add_dirt(local_id, ComponentDirt::PAINT, false))
}
