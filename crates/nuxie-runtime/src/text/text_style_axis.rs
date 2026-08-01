#[derive(Debug, Clone, Copy)]
struct StaticTextVariation {
    tag: u32,
    axis_local: usize,
    authored_value: f32,
}

/// Direct `TextStyleAxis::axisValueChanged` callback. The generated setter
/// dirties the retained TextStyle parent; its existing dirty chain reaches the
/// owning Text and embedded variation helper (`text_style_axis.cpp:27-30`).
pub(crate) fn text_style_axis_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStyleAxis")
        && property_key_for_name("TextStyleAxis", "axisValue") == Some(property_key))
    .then(|| {
        instance.component_parent_local(local_id).is_some_and(|style| {
            instance.add_dirt(style, crate::components::ComponentDirt::TEXT_SHAPE, false)
        })
    })
}

pub(crate) fn text_style_axis_uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStyleAxis")
        && property_key_for_name("TextStyleAxis", "tag") == Some(property_key))
    .then(|| {
        instance.component_parent_local(local_id).is_some_and(|style| {
            instance.add_dirt(style, crate::components::ComponentDirt::TEXT_SHAPE, false)
        })
    })
}
