use crate::{artboard::ArtboardInstance, text_style_owner};

/// `TextStylePaint` inherits TextStyle's shaping metrics but retains its own
/// paint/backend identity. Paint-only callbacks stay with the shape-paint
/// owner; this file owns the inherited shaping callback.
pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStylePaint"))
        .then(|| text_style_owner::metric_property_changed(instance, local_id, property_key))
        .flatten()
}
