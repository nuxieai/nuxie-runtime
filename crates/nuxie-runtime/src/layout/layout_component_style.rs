use crate::{
    artboard::ArtboardInstance, components::ComponentDirt, layout_component,
    properties::property_key_for_name,
};

const NODE_DIRTY_PROPERTIES: &[&str] = &[
    "layoutAlignmentType",
    "layoutWidthScaleType",
    "layoutHeightScaleType",
    "displayValue",
    "overflowValue",
    "intrinsicallySizedValue",
    "layoutTypeValue",
    "justifyItemsValue",
    "justifySelfValue",
    "flexWrapValue",
    "positionTypeValue",
    "flexDirectionValue",
    "directionValue",
    "flexBasis",
    "aspectRatio",
    "gapHorizontal",
    "gapVertical",
    "maxWidth",
    "maxHeight",
    "minWidth",
    "minHeight",
    "borderLeft",
    "borderRight",
    "borderTop",
    "borderBottom",
    "marginLeft",
    "marginRight",
    "marginTop",
    "marginBottom",
    "paddingLeft",
    "paddingRight",
    "paddingTop",
    "paddingBottom",
    "positionLeft",
    "positionRight",
    "positionTop",
    "positionBottom",
    "widthUnitsValue",
    "heightUnitsValue",
    "gapHorizontalUnitsValue",
    "gapVerticalUnitsValue",
    "maxWidthUnitsValue",
    "maxHeightUnitsValue",
    "minWidthUnitsValue",
    "minHeightUnitsValue",
    "borderLeftUnitsValue",
    "borderRightUnitsValue",
    "borderTopUnitsValue",
    "borderBottomUnitsValue",
    "marginLeftUnitsValue",
    "marginRightUnitsValue",
    "marginTopUnitsValue",
    "marginBottomUnitsValue",
    "paddingLeftUnitsValue",
    "paddingRightUnitsValue",
    "paddingTopUnitsValue",
    "paddingBottomUnitsValue",
    "positionLeftUnitsValue",
    "positionRightUnitsValue",
    "positionTopUnitsValue",
    "positionBottomUnitsValue",
];

const STYLE_DIRTY_PROPERTIES: &[&str] = &[
    "interpolationTime",
    "cornerRadiusTL",
    "cornerRadiusTR",
    "cornerRadiusBL",
    "cornerRadiusBR",
];

fn mark_parent_layout(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    let Some(parent_local) = instance.component_parent_local(style_local_id) else {
        return false;
    };
    layout_component::mark_layout_node_dirty(instance, parent_local)
}

fn parent_layout_local(instance: &ArtboardInstance, style_local_id: usize) -> Option<usize> {
    instance
        .component_parent_local(style_local_id)
        .filter(|local_id| {
            instance
                .component(*local_id)
                .is_some_and(|component| component.concrete.layout.is_some())
        })
}

fn scale_type_changed(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    let is_hug = ["layoutWidthScaleType", "layoutHeightScaleType"]
        .into_iter()
        .filter_map(|name| property_key_for_name("LayoutComponentStyle", name))
        .any(|key| instance.uint_property(style_local_id, key) == Some(2));
    let intrinsic_changed =
        property_key_for_name("LayoutComponentStyle", "intrinsicallySizedValue")
            .is_some_and(|key| instance.set_bool_property(style_local_id, key, is_hug));
    mark_parent_layout(instance, style_local_id) | intrinsic_changed
}

fn position_type_changed(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    let Some(parent_local) = parent_layout_local(instance, style_local_id) else {
        return false;
    };
    let Some(layout) = instance
        .component(parent_local)
        .and_then(|component| component.concrete.layout.as_ref())
    else {
        return false;
    };
    let position = layout.position();
    let preserve_left = layout.position_left_changed();
    let preserve_top = layout.position_top_changed();
    let absolute = property_key_for_name("LayoutComponentStyle", "positionTypeValue")
        .and_then(|key| instance.uint_property(style_local_id, key))
        == Some(2);

    let mut changed = false;
    for (name, value) in [
        ("positionLeft", if absolute { position.0 } else { 0.0 }),
        ("positionTop", if absolute { position.1 } else { 0.0 }),
        ("positionRight", 0.0),
        ("positionBottom", 0.0),
    ] {
        if absolute
            && ((name == "positionLeft" && preserve_left)
                || (name == "positionTop" && preserve_top))
        {
            continue;
        }
        if let Some(key) = property_key_for_name("LayoutComponentStyle", name) {
            changed |= instance.set_double_property(style_local_id, key, value);
        }
    }
    for (name, unit) in [
        ("positionLeftUnitsValue", u64::from(absolute)),
        ("positionTopUnitsValue", u64::from(absolute)),
        ("positionRightUnitsValue", 0),
        ("positionBottomUnitsValue", 0),
    ] {
        if let Some(key) = property_key_for_name("LayoutComponentStyle", name) {
            changed |= instance.set_uint_property(style_local_id, key, unit);
        }
    }
    layout_component::mark_layout_node_dirty(instance, parent_local) | changed
}

fn flex_direction_changed(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    parent_layout_local(instance, style_local_id).is_some_and(|parent_local| {
        layout_component::flex_direction_changed(instance, parent_local)
    })
}

// Ported from 4ac7b327 `LayoutComponentStyle::layoutTypeValueChanged`,
// which routes through `LayoutComponent::layoutTypeChanged`.
fn layout_type_changed(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    parent_layout_local(instance, style_local_id)
        .is_some_and(|parent_local| layout_component::layout_type_changed(instance, parent_local))
}

fn direction_changed(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    parent_layout_local(instance, style_local_id)
        .is_some_and(|parent_local| layout_component::direction_changed(instance, parent_local))
}

fn mark_parent_layout_style(instance: &mut ArtboardInstance, style_local_id: usize) -> bool {
    instance
        .component_parent_local(style_local_id)
        .is_some_and(|parent_local| {
            instance.add_dirt(parent_local, ComponentDirt::LAYOUT_STYLE, false)
        })
}

fn changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("LayoutComponentStyle") {
        return None;
    }
    if NODE_DIRTY_PROPERTIES
        .iter()
        .any(|name| property_key_for_name("LayoutComponentStyle", name) == Some(property_key))
    {
        return Some(mark_parent_layout(instance, local_id));
    }
    STYLE_DIRTY_PROPERTIES
        .iter()
        .any(|name| property_key_for_name("LayoutComponentStyle", name) == Some(property_key))
        .then(|| mark_parent_layout_style(instance, local_id))
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name == Some("LayoutComponentStyle") {
        if property_key_for_name("LayoutComponentStyle", "positionLeft") == Some(property_key) {
            if let Some(parent_local) = parent_layout_local(instance, local_id) {
                layout_component::mark_position_left_changed(instance, parent_local);
            }
        } else if property_key_for_name("LayoutComponentStyle", "positionTop") == Some(property_key)
            && let Some(parent_local) = parent_layout_local(instance, local_id)
        {
            layout_component::mark_position_top_changed(instance, parent_local);
        }
    }
    changed(instance, local_id, type_name, property_key)
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name == Some("LayoutComponentStyle") {
        if ["layoutWidthScaleType", "layoutHeightScaleType"]
            .into_iter()
            .any(|name| property_key_for_name("LayoutComponentStyle", name) == Some(property_key))
        {
            return Some(scale_type_changed(instance, local_id));
        }
        if property_key_for_name("LayoutComponentStyle", "layoutTypeValue") == Some(property_key) {
            return Some(layout_type_changed(instance, local_id));
        }
        if property_key_for_name("LayoutComponentStyle", "positionTypeValue") == Some(property_key)
        {
            return Some(position_type_changed(instance, local_id));
        }
        if property_key_for_name("LayoutComponentStyle", "flexDirectionValue") == Some(property_key)
        {
            return Some(flex_direction_changed(instance, local_id));
        }
        if property_key_for_name("LayoutComponentStyle", "directionValue") == Some(property_key) {
            return Some(direction_changed(instance, local_id));
        }
    }
    changed(instance, local_id, type_name, property_key)
}

pub(crate) fn bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    changed(instance, local_id, type_name, property_key)
}
