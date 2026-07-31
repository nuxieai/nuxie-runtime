// Direct owner for pinned C++ `src/nested_artboard_origin.cpp`.

fn apply_nested_artboard_origin_override(
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
