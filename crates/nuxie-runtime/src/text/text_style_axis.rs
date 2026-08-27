#[derive(Debug, Clone, Copy)]
struct StaticTextVariation {
    tag: u32,
    axis_local: usize,
    authored_value: f32,
}

/// Direct `TextStyleAxis::onAddedDirty` body after Component Super has linked
/// the occurrence to its retained parent. The callback validates that exact
/// parent, then appends the axis to the style in authored traversal order.
pub(crate) fn text_style_axis_on_added_dirty(
    objects: &mut crate::objects::InstanceObjectArena,
    axis_local: usize,
    parent: crate::components::ComponentHandle,
    parent_type: &str,
) -> anyhow::Result<()> {
    if !nuxie_schema::definition_by_name(parent_type)
        .is_some_and(|definition| definition.is_a("TextStyle"))
    {
        anyhow::bail!(
            "TextStyleAxis local {} requires a direct TextStyle parent",
            axis_local
        );
    }
    objects
        .component(parent)
        .and_then(|parent| parent.concrete.text_style.as_ref())
        .expect("TextStyle occurrence state")
        .register_variation(axis_local);
    Ok(())
}

/// Direct `TextStyleAxis::axisValueChanged` body. The generated setter has
/// already stored the new value; this callback dirties only the retained
/// TextStyle parent before the shared property notification tail runs.
fn text_style_axis_axis_value_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
) -> Option<bool> {
    let style = instance.component_parent_local(local_id)?;
    Some(instance.add_dirt(
        style,
        crate::components::ComponentDirt::TEXT_SHAPE,
        false,
    ))
}

pub(crate) fn text_style_axis_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStyleAxis")
        && property_key_for_name("TextStyleAxis", "axisValue") == Some(property_key))
    .then(|| text_style_axis_axis_value_changed(instance, local_id))
    .flatten()
}

/// Direct `TextStyleAxis::tagChanged` body, with the same store/callback/
/// notification ordering as the generated uint setter.
fn text_style_axis_tag_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
) -> Option<bool> {
    let style = instance.component_parent_local(local_id)?;
    Some(instance.add_dirt(
        style,
        crate::components::ComponentDirt::TEXT_SHAPE,
        false,
    ))
}

pub(crate) fn text_style_axis_uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStyleAxis")
        && property_key_for_name("TextStyleAxis", "tag") == Some(property_key))
    .then(|| text_style_axis_tag_changed(instance, local_id))
    .flatten()
}
