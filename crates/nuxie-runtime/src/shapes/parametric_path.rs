use std::collections::BTreeSet;

use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["width", "height", "originX", "originY"]
        .into_iter()
        .any(|name| property_key_for_name("ParametricPath", name) == Some(property_key))
    {
        return None;
    }

    let mut changed = super::mark_path_dirty(artboard, local_id);
    // Cycle guard: a malformed Shape parent cycle would walk forever here
    // (C++ hangs). Terminate via the DependencySorter visited-set idiom (see
    // runtime_layout_ancestors in components.rs), as if the group boundary
    // was reached. Unreachable on any valid file.
    let mut visited = BTreeSet::new();
    let mut parent = artboard.component_parent_local(local_id);
    while let Some(parent_local) = parent {
        if !visited.insert(parent_local) {
            break;
        }
        let Some(definition) = artboard
            .runtime_object_type_name(parent_local)
            .and_then(nuxie_schema::definition_by_name)
        else {
            break;
        };
        if definition.is_a("LayoutComponent") {
            changed |= artboard.mark_layout_node_changed(parent_local);
            break;
        }
        if definition.is_a("Node") {
            if definition.is_a("Shape") {
                // `Path::shape()` is the first Shape ancestor. Reaching one
                // before another Node boundary therefore identifies the same
                // owner that pinned C++ permits the walk to cross.
                parent = artboard.component_parent_local(parent_local);
                continue;
            }
            break;
        }
        // Non-Node ContainerComponents do not form a group boundary.
        parent = artboard.component_parent_local(parent_local);
    }
    Some(changed)
}
