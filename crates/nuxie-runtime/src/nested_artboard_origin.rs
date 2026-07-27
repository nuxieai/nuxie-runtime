//! Runtime counterpart of pinned C++ `nested_artboard_origin.hpp/.cpp`.
//!
//! `Artboard` retains authored-order construction and generated-property
//! dispatch. This owner applies the optional origin override when a nested
//! child is mounted and reapplies it when either authored origin changes.

use super::{ArtboardInstance, ComponentDirt, InstanceObjectArena, property_key_for_name};

impl ArtboardInstance {
    pub(super) fn reapply_nested_artboard_origin(&mut self, local_id: usize) -> bool {
        let Some(host_local_id) = self.component_parent_local(local_id) else {
            return false;
        };
        let Some(origin_x_key) = property_key_for_name("Artboard", "originX") else {
            return false;
        };
        let Some(origin_y_key) = property_key_for_name("Artboard", "originY") else {
            return false;
        };
        let Some(origin_x) = property_key_for_name("NestedArtboardOrigin", "originX")
            .and_then(|key| self.double_property(local_id, key))
        else {
            return false;
        };
        let Some(origin_y) = property_key_for_name("NestedArtboardOrigin", "originY")
            .and_then(|key| self.double_property(local_id, key))
        else {
            return false;
        };
        let changed = self
            .nested_artboards
            .get_mut(&host_local_id)
            .is_some_and(|nested| {
                let mut changed = nested.child.set_double_property(0, origin_x_key, origin_x);
                changed |= nested.child.set_double_property(0, origin_y_key, origin_y);
                changed
            });
        if changed {
            self.add_dirt(host_local_id, ComponentDirt::TRANSFORM, false);
            self.add_dirt(host_local_id, ComponentDirt::WORLD_TRANSFORM, true);
        }
        changed
    }
}

pub(super) fn apply_nested_artboard_origin_override(
    parent_objects: &InstanceObjectArena,
    host_local_id: usize,
    child: &mut ArtboardInstance,
) -> bool {
    let Some(host) = parent_objects.component_handle(host_local_id) else {
        return false;
    };
    let origin_local = (0..parent_objects.child_len(host))
        .filter_map(|index| parent_objects.child_at(host, index))
        .filter_map(|child| parent_objects.component(child))
        .find(|component| component.type_name == "NestedArtboardOrigin")
        .map(|component| component.local_id);
    let Some(origin_local) = origin_local else {
        return false;
    };
    let Some(origin_x) = property_key_for_name("NestedArtboardOrigin", "originX")
        .and_then(|key| parent_objects.double_property(origin_local, key))
    else {
        return false;
    };
    let Some(origin_y) = property_key_for_name("NestedArtboardOrigin", "originY")
        .and_then(|key| parent_objects.double_property(origin_local, key))
    else {
        return false;
    };
    let Some(origin_x_key) = property_key_for_name("Artboard", "originX") else {
        return false;
    };
    let Some(origin_y_key) = property_key_for_name("Artboard", "originY") else {
        return false;
    };

    let mut changed = child.set_double_property(0, origin_x_key, origin_x);
    changed |= child.set_double_property(0, origin_y_key, origin_y);
    changed
}
