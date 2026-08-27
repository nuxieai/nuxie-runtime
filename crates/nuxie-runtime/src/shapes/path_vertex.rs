use crate::ArtboardInstance;

/// `PathVertex::markGeometryDirty`: the vertex dirties its concrete parent
/// Path, never itself. PointsPath additionally dirties its retained Skin.
pub(crate) fn mark_geometry_dirty(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    let Some(path_local) = artboard.component_parent_local(local_id) else {
        // Parametric paths own unparented virtual vertices.
        return false;
    };
    if artboard
        .component(path_local)
        .is_some_and(|component| component.type_name == "PointsPath")
    {
        super::points_path::mark_path_dirty(artboard, path_local, true)
    } else {
        super::mark_path_dirty(artboard, path_local)
    }
}
