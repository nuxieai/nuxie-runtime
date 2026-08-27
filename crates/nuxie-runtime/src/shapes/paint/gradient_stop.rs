use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

fn mark_retained_parent_gradient_dirty(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    dirt: ComponentDirt,
) -> bool {
    // `GradientStop::onAddedDirty` retains the Component parent before either
    // generated callback can run. A later `parentId` property write does not
    // replace that pointer in C++, so route through the occurrence relation
    // rather than re-reading the serialized property.
    let Some(parent_local) = artboard.component_parent_local(local_id) else {
        return false;
    };
    if !matches!(
        artboard.runtime_object_type_name(parent_local),
        Some("LinearGradient" | "RadialGradient")
    ) {
        return false;
    }
    artboard.add_dirt(parent_local, dirt, false)
}

fn color_value_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    mark_retained_parent_gradient_dirty(artboard, local_id, ComponentDirt::PAINT)
}

fn position_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    mark_retained_parent_gradient_dirty(
        artboard,
        local_id,
        ComponentDirt::PAINT | ComponentDirt::STOPS,
    )
}

pub(crate) fn color_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("GradientStop", "colorValue") != Some(property_key) {
        return None;
    }
    Some(color_value_changed(artboard, local_id))
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("GradientStop", "position") != Some(property_key) {
        return None;
    }
    Some(position_changed(artboard, local_id))
}
