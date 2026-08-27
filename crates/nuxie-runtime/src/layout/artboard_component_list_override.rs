use crate::{artboard::ArtboardInstance, properties::property_key_for_name};

const TYPE_NAME: &str = "ArtboardComponentListOverride";

fn key_for(name: &str) -> Option<u16> {
    property_key_for_name(TYPE_NAME, name)
}

// Keep the six generated property callbacks in this source owner so their
// width/height dispatch remains directly comparable with the pinned C++ pair.
// The retained add/remove and override traversal bodies remain packed in the
// protected artboard/draw owners and are tracked as a correspondence gap.
fn update_width_override(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance.mark_component_list_override_changed(local_id)
}

fn update_height_override(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance.mark_component_list_override_changed(local_id)
}

fn instance_width_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_width_override(instance, local_id)
}

fn instance_height_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_height_override(instance, local_id)
}

fn instance_width_units_value_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_width_override(instance, local_id)
}

fn instance_height_units_value_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_height_override(instance, local_id)
}

fn instance_width_scale_type_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_width_override(instance, local_id)
}

fn instance_height_scale_type_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    update_height_override(instance, local_id)
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some(TYPE_NAME) {
        return None;
    }
    if Some(property_key) == key_for("instanceWidth") {
        Some(instance_width_changed(instance, local_id))
    } else if Some(property_key) == key_for("instanceHeight") {
        Some(instance_height_changed(instance, local_id))
    } else {
        None
    }
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some(TYPE_NAME) {
        return None;
    }
    if Some(property_key) == key_for("instanceWidthUnitsValue") {
        Some(instance_width_units_value_changed(instance, local_id))
    } else if Some(property_key) == key_for("instanceHeightUnitsValue") {
        Some(instance_height_units_value_changed(instance, local_id))
    } else if Some(property_key) == key_for("instanceWidthScaleType") {
        Some(instance_width_scale_type_changed(instance, local_id))
    } else if Some(property_key) == key_for("instanceHeightScaleType") {
        Some(instance_height_scale_type_changed(instance, local_id))
    } else {
        None
    }
}
