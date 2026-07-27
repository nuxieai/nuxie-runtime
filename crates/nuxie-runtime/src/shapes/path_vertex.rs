//! Focused runtime bridge for pinned C++ `include/rive/shapes/path_vertex.hpp`
//! and `src/shapes/path_vertex.cpp`.

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};
use crate::objects::InstanceObjectArena;

/// Direct `PathVertex::onAddedDirty`: register this occurrence on its retained
/// parent Path in authored object order.
pub(crate) fn on_added_dirty(objects: &mut InstanceObjectArena, handle: ComponentHandle) -> bool {
    if !objects.component(handle).is_some_and(|component| {
        matches!(
            component.type_name,
            "StraightVertex"
                | "CubicMirroredVertex"
                | "CubicAsymmetricVertex"
                | "CubicDetachedVertex"
        )
    }) {
        return false;
    }
    super::vertex::register_on_parent_skinnable(objects, handle);
    true
}

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
