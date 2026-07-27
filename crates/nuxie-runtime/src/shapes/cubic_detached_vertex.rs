//! Direct Rust owner for pinned C++
//! `include/rive/shapes/cubic_detached_vertex.hpp` and
//! `src/shapes/cubic_detached_vertex.cpp`.

use crate::ArtboardInstance;
use crate::properties::property_key_for_name;

/// Direct in/out rotation and distance callbacks; x/y route through
/// CubicVertex.
pub(crate) fn apply_double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    if type_name != Some("CubicDetachedVertex")
        || !["inRotation", "inDistance", "outRotation", "outDistance"]
            .iter()
            .any(|name| property_key_for_name("CubicDetachedVertex", name) == Some(property_key))
    {
        return false;
    }
    super::path_vertex::mark_geometry_dirty(artboard, local_id);
    true
}
