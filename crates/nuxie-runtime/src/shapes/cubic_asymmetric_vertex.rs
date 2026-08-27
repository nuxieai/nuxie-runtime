use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if let inherited @ Some(_) = super::cubic_vertex::position_property_changed(
        artboard,
        local_id,
        "CubicAsymmetricVertex",
        property_key,
    ) {
        return inherited;
    }
    if property_key_for_name("CubicAsymmetricVertex", "rotation") == Some(property_key) {
        return Some(rotation_changed(artboard, local_id));
    }
    if property_key_for_name("CubicAsymmetricVertex", "inDistance") == Some(property_key) {
        return Some(in_distance_changed(artboard, local_id));
    }
    if property_key_for_name("CubicAsymmetricVertex", "outDistance") == Some(property_key) {
        return Some(out_distance_changed(artboard, local_id));
    }
    None
}

fn rotation_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    // Rust rebuilds both control points directly from retained scalar fields,
    // so there are no `m_InValid`/`m_OutValid` cache bits to clear.
    super::path_vertex::mark_geometry_dirty(artboard, local_id)
}

fn in_distance_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    // The source's input-cache invalidation is represented by rebuilding the
    // input control point when the parent Path consumes this dirt.
    super::path_vertex::mark_geometry_dirty(artboard, local_id)
}

fn out_distance_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    // The source's output-cache invalidation is represented by rebuilding the
    // output control point when the parent Path consumes this dirt.
    super::path_vertex::mark_geometry_dirty(artboard, local_id)
}
