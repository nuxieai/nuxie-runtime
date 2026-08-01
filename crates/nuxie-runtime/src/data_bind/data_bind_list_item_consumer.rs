//! Structural list dependency owner matching C++ `DataBindListItemConsumer`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDataBindListItemConsumer {
    ArtboardComponentList,
    ListPath,
}

/// Exact `DataBindListItemConsumer::from(Core*)` boundary. C++ dispatches on
/// the target core type and does not consult the data bind's property key.
pub(crate) fn from_target(
    type_name: &str,
    _property_key: Option<u64>,
) -> Option<RuntimeDataBindListItemConsumer> {
    match type_name {
        "ArtboardComponentList" => Some(RuntimeDataBindListItemConsumer::ArtboardComponentList),
        "ListPath" => Some(RuntimeDataBindListItemConsumer::ListPath),
        _ => None,
    }
}

pub(crate) fn changed(previous_item_count: usize, next_item_count: usize) -> bool {
    crate::context_value_list::item_count_changed(previous_item_count, next_item_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_path_consumer_is_type_dispatched_and_property_key_agnostic() {
        assert_eq!(
            from_target("ListPath", Some(874)),
            Some(RuntimeDataBindListItemConsumer::ListPath)
        );
        assert_eq!(
            from_target("ListPath", Some(873)),
            Some(RuntimeDataBindListItemConsumer::ListPath)
        );
        assert_eq!(
            from_target("ListPath", None),
            Some(RuntimeDataBindListItemConsumer::ListPath)
        );
        assert_eq!(from_target("Unknown", Some(874)), None);
    }
}
