use nuxie_schema::definition_by_name;

use crate::{artboard::ArtboardInstance, properties::property_key_for_name, text_owner};

pub(super) fn owning_text(instance: &ArtboardInstance, local_id: usize) -> Option<usize> {
    instance
        .component(local_id)?
        .concrete
        .text_style
        .as_ref()?
        .text()
        .and_then(|text| instance.component_local_id(text))
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if !type_name.is_some_and(|type_name| {
        definition_by_name(type_name).is_some_and(|definition| definition.is_a("TextStyle"))
    }) {
        return None;
    }
    metric_property_changed(instance, local_id, property_key)
}

pub(super) fn metric_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    let shapes_text = ["fontSize", "lineHeight", "letterSpacing"]
        .into_iter()
        .any(|name| property_key_for_name("TextStyle", name) == Some(property_key));
    shapes_text.then(|| {
        owning_text(instance, local_id)
            .is_some_and(|text| text_owner::mark_shape_dirty(instance, text))
    })
}
