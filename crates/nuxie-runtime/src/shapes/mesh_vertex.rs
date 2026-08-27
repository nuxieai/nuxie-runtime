use crate::ArtboardInstance;

/// Direct `MeshVertex::onAddedDirty` after `Vertex::onAddedDirty` has linked
/// the retained parent: validate the concrete Mesh parent, then register the
/// vertex in authored child order.
pub(crate) fn on_added_dirty_after_super(
    mesh_local: usize,
    vertices: &mut Vec<usize>,
    vertex_local: usize,
    parent: Option<(usize, &'static str)>,
) -> Option<()> {
    (parent == Some((mesh_local, "Mesh"))).then(|| vertices.push(vertex_local))
}

/// `MeshVertex::markGeometryDirty` delegates to its concrete Mesh parent.
pub(crate) fn geometry_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance
        .component_parent_local(local_id)
        .filter(|mesh_local| instance.runtime_object_type_name(*mesh_local) == Some("Mesh"))
        .is_some_and(|mesh_local| super::mesh::mark_vertices_dirty(instance, mesh_local))
}
