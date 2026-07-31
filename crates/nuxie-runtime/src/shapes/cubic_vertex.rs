use crate::ArtboardInstance;

/// `CubicVertex::{xChanged,yChanged}` invalidate both lazily computed control
/// points after the base Vertex callback dirties geometry. Control points are
/// derived directly while rebuilding Rust's retained path, so no second cache
/// is needed here.
pub(crate) fn position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_key: u16,
) -> Option<bool> {
    super::vertex::position_property_changed(artboard, local_id, type_name, property_key)
}
