use crate::{artboard::ArtboardInstance, properties::property_key_for_name};

fn affects_hosted_layout(property_key: u16) -> bool {
    [
        "instanceWidth",
        "instanceHeight",
        "instanceWidthUnitsValue",
        "instanceHeightUnitsValue",
        "instanceWidthScaleType",
        "instanceHeightScaleType",
    ]
    .into_iter()
    .any(|name| property_key_for_name("ArtboardComponentListOverride", name) == Some(property_key))
}

fn changed(instance: &mut ArtboardInstance, local_id: usize, property_key: u16) -> Option<bool> {
    affects_hosted_layout(property_key)
        .then(|| instance.mark_component_list_override_changed(local_id))
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("ArtboardComponentListOverride"))
        .then(|| changed(instance, local_id, property_key))
        .flatten()
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    double_property_changed(instance, local_id, type_name, property_key)
}
