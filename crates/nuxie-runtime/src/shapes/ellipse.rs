use crate::ArtboardInstance;

/// Ellipse has no leaf generated changed callbacks; inherited ParametricPath
/// owns width/height/origin invalidation.
pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    super::parametric_path::property_changed(artboard, local_id, property_key)
}
