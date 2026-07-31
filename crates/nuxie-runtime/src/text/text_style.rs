use crate::{artboard::ArtboardInstance, properties::property_key_for_name, text_owner};

pub(super) fn owning_text(instance: &ArtboardInstance, mut local_id: usize) -> Option<usize> {
    loop {
        local_id = instance.component_parent_local(local_id)?;
        if matches!(
            instance
                .component(local_id)
                .map(|component| component.type_name),
            Some("Text" | "TextInput")
        ) {
            return Some(local_id);
        }
    }
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("TextStyle") {
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
