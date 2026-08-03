use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

/// Inner feathering is fill-only even when the serialized flag survives a
/// paint conversion to Stroke.
pub(crate) fn is_inner(authored_inner: bool, parent_type_name: Option<&str>) -> bool {
    authored_inner && parent_type_name == Some("Fill")
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["strength", "offsetX", "offsetY"]
        .into_iter()
        .any(|name| property_key_for_name("Feather", name) == Some(property_key))
    {
        return None;
    }
    let authored_inner = property_key_for_name("Feather", "inner")
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(false);
    let parent_type_name = artboard
        .component_parent_local(local_id)
        .and_then(|parent| artboard.runtime_object_type_name(parent));
    let inner = is_inner(authored_inner, parent_type_name);
    let dirt = if inner {
        ComponentDirt::PAINT | ComponentDirt::WORLD_TRANSFORM
    } else {
        ComponentDirt::PAINT
    };
    Some(artboard.add_dirt(local_id, dirt, false))
}

// Pinned Feather overrides no generated callback for `inner` or `spaceValue`.
pub(crate) fn bool_property_changed(
    _artboard: &mut ArtboardInstance,
    _local_id: usize,
    _property_key: u16,
) -> Option<bool> {
    // The generated setter reaches an inherited empty callback. Treat it as
    // handled so no generic cache refresh is substituted for that no-op.
    Some(false)
}

pub(crate) fn uint_property_changed(
    _artboard: &mut ArtboardInstance,
    _local_id: usize,
    _property_key: u16,
) -> Option<bool> {
    Some(false)
}

#[cfg(test)]
mod tests {
    use super::is_inner;

    #[test]
    fn serialized_inner_feather_is_effective_only_on_a_fill() {
        assert!(is_inner(true, Some("Fill")));
        assert!(!is_inner(true, Some("Stroke")));
        assert!(!is_inner(true, None));
        assert!(!is_inner(false, Some("Fill")));
    }
}
