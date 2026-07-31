use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if let inherited @ Some(_) = super::polygon::property_changed(artboard, local_id, property_key)
    {
        return inherited;
    }
    if property_key_for_name("Star", "innerRadius") != Some(property_key) {
        return None;
    }
    Some(super::mark_path_dirty(artboard, local_id))
}
