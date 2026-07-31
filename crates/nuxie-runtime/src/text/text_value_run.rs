use crate::{artboard::ArtboardInstance, properties::property_key_for_name, text_owner};

/// Direct `TextValueRun::textChanged -> Text::markShapeDirty` callback.
pub(crate) fn string_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextValueRun")
        && property_key_for_name("TextValueRun", "text") == Some(property_key))
    .then(|| {
        let Some(text_local) = instance.component_parent_local(local_id) else {
            return false;
        };
        if instance
            .component(text_local)
            .is_none_or(|component| component.type_name != "Text")
        {
            return false;
        }
        text_owner::mark_shape_dirty(instance, text_local)
    })
}

/// Direct `TextValueRun::styleIdChanged`: only a resolved TextStylePaint is
/// installed, and successful installation invalidates the owning Text shape.
pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextValueRun")
        && property_key_for_name("TextValueRun", "styleId") == Some(property_key))
    .then(|| {
        let Some(style_local) = instance
            .uint_property(local_id, property_key)
            .and_then(|value| usize::try_from(value).ok())
        else {
            return false;
        };
        if instance
            .component(style_local)
            .is_none_or(|component| component.type_name != "TextStylePaint")
        {
            return false;
        }
        let Some(text_local) = instance.component_parent_local(local_id) else {
            return false;
        };
        if instance
            .component(text_local)
            .is_none_or(|component| component.type_name != "Text")
        {
            return false;
        }
        text_owner::mark_shape_dirty(instance, text_local)
    })
}
