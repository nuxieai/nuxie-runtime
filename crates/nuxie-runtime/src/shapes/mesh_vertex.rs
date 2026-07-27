//! Direct Rust owner for pinned C++ `include/rive/shapes/mesh_vertex.hpp` and
//! `src/shapes/mesh_vertex.cpp`.

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;
use nuxie_schema::definition_by_name;

fn is_mesh_vertex(type_name: &str) -> bool {
    definition_by_name(type_name).is_some_and(|definition| definition.is_a("MeshVertex"))
}

/// Direct `MeshVertex::onAddedDirty`: register this occurrence on its retained
/// parent Mesh in authored object order. Generated subclasses such as
/// `ContourMeshVertex` inherit this exact override in pinned C++.
pub(crate) fn on_added_dirty(objects: &mut InstanceObjectArena, handle: ComponentHandle) -> bool {
    if objects
        .component(handle)
        .is_none_or(|component| !is_mesh_vertex(component.type_name))
    {
        return false;
    }
    let parent = objects
        .component(handle)
        .and_then(|component| component.parent);
    if let Some(skinnable) = parent.and_then(|parent| {
        objects
            .component_mut(parent)
            .and_then(|parent| parent.concrete.skinnable.as_mut())
    }) {
        skinnable.vertices.push(handle);
    }
    true
}

/// Direct `MeshVertex::markGeometryDirty` reached by inherited Vertex x/y
/// callbacks.
pub(crate) fn apply_position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    if type_name != Some("MeshVertex")
        || !["x", "y"]
            .iter()
            .any(|name| property_key_for_name("Vertex", name) == Some(property_key))
    {
        return false;
    }
    let Some(mesh_local) = artboard.component_parent_local(local_id) else {
        return false;
    };
    let Some(mesh) = artboard.component_handle(mesh_local) else {
        return false;
    };
    if let Some(skin) = artboard
        .objects
        .component(mesh)
        .and_then(|component| component.concrete.skinnable.as_ref())
        .and_then(|skinnable| skinnable.skin)
    {
        artboard.add_component_dirt(skin, ComponentDirt::SKIN, false);
    }
    artboard.add_component_dirt(mesh, ComponentDirt::VERTICES, false)
}
