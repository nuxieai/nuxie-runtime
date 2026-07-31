use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if let inherited @ Some(_) = super::cubic_vertex::position_property_changed(
        artboard,
        local_id,
        "CubicMirroredVertex",
        property_key,
    ) {
        return inherited;
    }
    if !["rotation", "distance"]
        .into_iter()
        .any(|name| property_key_for_name("CubicMirroredVertex", name) == Some(property_key))
    {
        return None;
    }
    Some(super::path_vertex::mark_geometry_dirty(artboard, local_id))
}
