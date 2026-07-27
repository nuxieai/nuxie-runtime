//! Direct Rust owner for pinned C++ `include/rive/shapes/straight_vertex.hpp`
//! and `src/shapes/straight_vertex.cpp`.

use crate::ArtboardInstance;
use crate::properties::property_key_for_name;

/// Direct `StraightVertex::radiusChanged`.
pub(crate) fn apply_double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    if type_name != Some("StraightVertex")
        || property_key_for_name("StraightVertex", "radius") != Some(property_key)
    {
        return false;
    }
    super::path_vertex::mark_geometry_dirty(artboard, local_id);
    true
}
