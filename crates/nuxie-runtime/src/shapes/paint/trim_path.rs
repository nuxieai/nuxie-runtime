use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["start", "end", "offset"]
        .into_iter()
        .any(|name| property_key_for_name("TrimPath", name) == Some(property_key))
    {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("TrimPath", "modeValue") != Some(property_key) {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}
