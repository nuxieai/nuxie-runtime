use crate::{ArtboardInstance, ComponentDirt};

/// `PathVertex::markGeometryDirty`: the vertex dirties its concrete parent
/// Path, never itself. PointsPath additionally dirties its retained Skin.
pub(crate) fn mark_geometry_dirty(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    let Some(path_local) = artboard.component_parent_local(local_id) else {
        // Parametric paths own unparented virtual vertices.
        return false;
    };
    let mut changed = artboard.mark_points_path_skin_dirty(path_local);
    changed |= artboard.add_dirt(path_local, ComponentDirt::PATH, false);
    changed
}
