use crate::ArtboardInstance;
use nuxie_graph::MeshVertexNode;

/// Direct `MeshVertex::onAddedDirty`: validate the concrete Mesh parent and
/// register the vertex in authored child order.
pub(crate) fn on_added_dirty(
    mesh_local: usize,
    vertices: &mut Vec<usize>,
    vertex: &MeshVertexNode,
    parent_local: Option<usize>,
) -> Option<()> {
    (matches!(vertex.type_name, "MeshVertex" | "ContourMeshVertex")
        && parent_local == Some(mesh_local))
    .then(|| {
        vertices.push(vertex.local_id);
    })
}

/// `MeshVertex::markGeometryDirty` delegates to its concrete Mesh parent.
pub(crate) fn geometry_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance
        .component_parent_local(local_id)
        .is_some_and(|mesh_local| super::mesh::mark_vertices_dirty(instance, mesh_local))
}
