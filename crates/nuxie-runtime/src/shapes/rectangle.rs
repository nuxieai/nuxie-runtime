use crate::{ArtboardInstance, properties::property_key_for_name};

/// The generated `RectangleBase::linkCornerRadiusChanged` callback is empty.
///
/// Returning an owned no-op keeps the generic property fallback from
/// invalidating prepared output for a setter that pinned C++ only publishes
/// through `notifyPropertyChanged`.
pub(crate) fn bool_property_changed(property_key: u16) -> Option<bool> {
    (property_key_for_name("Rectangle", "linkCornerRadius") == Some(property_key)).then_some(false)
}

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if let inherited @ Some(_) =
        super::parametric_path::property_changed(artboard, local_id, property_key)
    {
        return inherited;
    }
    if ![
        "cornerRadiusTL",
        "cornerRadiusTR",
        "cornerRadiusBL",
        "cornerRadiusBR",
    ]
    .into_iter()
    .any(|name| property_key_for_name("Rectangle", name) == Some(property_key))
    {
        return None;
    }
    Some(super::mark_path_dirty(artboard, local_id))
}
