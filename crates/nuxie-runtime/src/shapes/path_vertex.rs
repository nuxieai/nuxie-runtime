//! Focused runtime bridge for pinned C++ `include/rive/shapes/path_vertex.hpp`
//! and `src/shapes/path_vertex.cpp`.

use crate::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;

/// Literal `PathVertex::markGeometryDirty`: dirty the optional retained Skin,
/// then the owning Path.
pub(crate) fn mark_geometry_dirty(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    let Some(path_local) = artboard.component_parent_local(local_id) else {
        return false;
    };
    let Some(path) = artboard.component_handle(path_local) else {
        return false;
    };
    if let Some(skin) = artboard
        .objects
        .component(path)
        .and_then(|component| component.concrete.skinnable.as_ref())
        .and_then(|skinnable| skinnable.skin)
    {
        artboard.add_component_dirt(skin, ComponentDirt::SKIN, false);
    }
    artboard.add_component_dirt(path, ComponentDirt::PATH, false);
    true
}

/// Temporary dispatch for the concrete derived-vertex callbacks whose own
/// focused files are queued in the live-draw wave. Base x/y callbacks are
/// already routed through `vertex.rs` / `cubic_vertex.rs`.
pub(crate) fn apply_derived_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    let Some(
        type_name @ ("StraightVertex"
        | "CubicMirroredVertex"
        | "CubicAsymmetricVertex"
        | "CubicDetachedVertex"),
    ) = type_name
    else {
        return false;
    };

    let properties: &[&str] = match type_name {
        "StraightVertex" => &["radius"],
        "CubicMirroredVertex" => &["rotation", "distance"],
        "CubicAsymmetricVertex" => &["rotation", "inDistance", "outDistance"],
        "CubicDetachedVertex" => &["inRotation", "inDistance", "outRotation", "outDistance"],
        _ => unreachable!("path-vertex type was filtered above"),
    };
    if !properties
        .iter()
        .any(|name| property_key_for_name(type_name, name) == Some(property_key))
    {
        return false;
    }
    mark_geometry_dirty(artboard, local_id);
    true
}
