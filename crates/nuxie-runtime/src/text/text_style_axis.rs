#[derive(Debug, Clone, Copy)]
struct StaticTextVariation {
    tag: u32,
    axis_local: usize,
    authored_value: f32,
}

/// Direct `TextStyleAxis::axisValueChanged` callback. The generated setter
/// dirties only the retained TextStyle parent (`text_style_axis.cpp:27-30`);
/// the TextStyle pair owns the later Text/helper cascade.
pub(crate) fn text_style_axis_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStyleAxis")
        && property_key_for_name("TextStyleAxis", "axisValue") == Some(property_key))
    .then(|| {
        let style = instance.component_parent_local(local_id)?;
        instance
            .component(style)
            .is_some_and(|style| {
                definition_by_name(style.type_name)
                    .is_some_and(|definition| definition.is_a("TextStyle"))
            })
            .then(|| instance.add_dirt(style, crate::components::ComponentDirt::TEXT_SHAPE, false))
    })
    .flatten()
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
        let style = instance.component_parent_local(local_id)?;
        instance
            .component(style)
            .is_some_and(|style| {
                definition_by_name(style.type_name)
                    .is_some_and(|definition| definition.is_a("TextStyle"))
            })
            .then(|| instance.add_dirt(style, crate::components::ComponentDirt::TEXT_SHAPE, false))
    })
    .flatten()
}
