use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("DashPath", "offset") != Some(property_key) {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("DashPath", "offsetIsPercentage") != Some(property_key) {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}
