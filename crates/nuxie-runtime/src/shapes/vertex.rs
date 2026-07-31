use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_key: u16,
) -> Option<bool> {
    if !["x", "y"]
        .into_iter()
        .any(|name| property_key_for_name(type_name, name) == Some(property_key))
    {
        return None;
    }
    Some(super::path_vertex::mark_geometry_dirty(artboard, local_id))
}
