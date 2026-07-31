use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

pub(crate) fn color_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("GradientStop", "colorValue") != Some(property_key) {
        return None;
    }
    Some(artboard.mark_parent_gradient_dirty(local_id, ComponentDirt::PAINT))
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("GradientStop", "position") != Some(property_key) {
        return None;
    }
    Some(artboard.mark_parent_gradient_dirty(local_id, ComponentDirt::PAINT | ComponentDirt::STOPS))
}
