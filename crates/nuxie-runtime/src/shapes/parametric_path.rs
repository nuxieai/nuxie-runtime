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
    let mut parent = artboard.component_parent_local(local_id);
    while let Some(parent_local) = parent {
        match artboard.runtime_object_type_name(parent_local) {
            Some("LayoutComponent") => {
                changed |= artboard.mark_layout_node_changed(parent_local);
                break;
            }
            Some("Shape") => {
                parent = artboard.component_parent_local(parent_local);
            }
            // A non-Shape Node (and any malformed parent) is the group
            // boundary in pinned `markPathDirty(sendToLayout=true)`.
            _ => break,
        }
    }
    Some(changed)
}
