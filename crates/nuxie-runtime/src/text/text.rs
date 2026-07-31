use crate::{
    artboard::ArtboardInstance, components::ComponentDirt, layout_node_provider,
    properties::property_key_for_name,
};

pub(crate) fn mark_shape_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    mark_shape_dirty_with_layout(instance, text_local_id, true)
}

pub(crate) fn mark_shape_dirty_without_layout(
    instance: &mut ArtboardInstance,
    text_local_id: usize,
) -> bool {
    mark_shape_dirty_with_layout(instance, text_local_id, false)
}

fn mark_shape_dirty_with_layout(
    instance: &mut ArtboardInstance,
    text_local_id: usize,
    send_to_layout: bool,
) -> bool {
    if !matches!(
        instance
            .component(text_local_id)
            .map(|component| component.type_name),
        Some("Text" | "TextInput")
    ) {
        return false;
    }
    if let Some(text) = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
    {
        text.invalidate_bounds();
    }
    let mut changed = instance.add_dirt(text_local_id, ComponentDirt::PATH, false);
    changed |= instance.add_dirt(text_local_id, ComponentDirt::WORLD_TRANSFORM, true);
    if send_to_layout {
        changed |= layout_node_provider::mark_layout_node_dirty(instance, text_local_id);
    }
    changed
}

fn effective_sizing(instance: &ArtboardInstance, text_local_id: usize) -> u64 {
    let authored = property_key_for_name("Text", "sizingValue")
        .and_then(|key| instance.uint_property(text_local_id, key))
        .unwrap_or(0);
    instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
        .map_or(authored, |text| text.effective_sizing(authored))
}

fn mark_paint_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    instance.add_dirt(text_local_id, ComponentDirt::PAINT, false)
}

fn mark_origin_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    mark_paint_dirty(instance, text_local_id)
        | instance.add_dirt(text_local_id, ComponentDirt::WORLD_TRANSFORM, true)
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("Text") {
        return None;
    }
    if property_key_for_name("Text", "width") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) != 0 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if property_key_for_name("Text", "height") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) == 2 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if property_key_for_name("Text", "paragraphSpacing") == Some(property_key) {
        return Some(mark_paint_dirty(instance, local_id));
    }
    ["originX", "originY"]
        .into_iter()
        .any(|name| property_key_for_name("Text", name) == Some(property_key))
        .then(|| mark_origin_dirty(instance, local_id))
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("Text") {
        return None;
    }
    if property_key_for_name("Text", "overflowValue") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) != 0 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if ["alignValue", "sizingValue", "verticalTrimValue"]
        .into_iter()
        .any(|name| property_key_for_name("Text", name) == Some(property_key))
    {
        return Some(mark_shape_dirty(instance, local_id));
    }
    (property_key_for_name("Text", "originValue") == Some(property_key))
        .then(|| mark_origin_dirty(instance, local_id))
}
