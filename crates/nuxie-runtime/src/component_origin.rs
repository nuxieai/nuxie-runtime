// Direct owner for pinned C++ `src/component_origin.cpp`.

fn component_origin_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("ComponentOrigin")
        || (property_key_for_name("ComponentOrigin", "originX") != Some(property_key)
            && property_key_for_name("ComponentOrigin", "originY") != Some(property_key))
    {
        return None;
    }

    let host_local_id = instance.component_parent_local(local_id)?;
    if instance
        .component(host_local_id)
        .is_some_and(|host| host.type_name == "LayoutComponent")
    {
        return Some(instance.add_dirt(
            host_local_id,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH,
            true,
        ));
    }

    let origin_x_key = property_key_for_name("Artboard", "originX")?;
    let origin_y_key = property_key_for_name("Artboard", "originY")?;
    let origin_x = property_key_for_name("ComponentOrigin", "originX")
        .and_then(|key| instance.double_property(local_id, key))?;
    let origin_y = property_key_for_name("ComponentOrigin", "originY")
        .and_then(|key| instance.double_property(local_id, key))?;
    let changed = instance
        .nested_artboards
        .get_mut(&host_local_id)
        .is_some_and(|nested| {
            let mut changed = nested
                .child
                .set_double_property(0, origin_x_key, origin_x);
            changed |= nested
                .child
                .set_double_property(0, origin_y_key, origin_y);
            changed
        });
    if changed {
        instance.add_dirt(host_local_id, ComponentDirt::TRANSFORM, false);
        instance.add_dirt(host_local_id, ComponentDirt::WORLD_TRANSFORM, true);
    }
    Some(changed)
}

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
        .find(|component| component.type_name == "ComponentOrigin")
        .map(|component| component.local_id);
    let Some(origin_local) = origin_local else {
        return false;
    };
    let Some(origin_x) = property_key_for_name("ComponentOrigin", "originX")
        .and_then(|key| parent_objects.double_property(origin_local, key))
    else {
        return false;
    };
    let Some(origin_y) = property_key_for_name("ComponentOrigin", "originY")
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
