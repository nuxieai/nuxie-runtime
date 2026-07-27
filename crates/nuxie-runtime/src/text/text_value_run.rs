//! Direct Rust home for `include/rive/text/text_value_run.hpp` and
//! `src/text/text_value_run.cpp` behavior owned by the retained Artboard.

use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, InstanceSlot};

pub(crate) fn set_root_text_value_run(
    instance: &mut ArtboardInstance,
    name: &str,
    value: Vec<u8>,
) -> Option<bool> {
    let text_property_key = property_key_for_name("TextValueRun", "text")?;
    let local_id = root_text_value_run_local_id(instance.slots(), name)?;
    if instance.string_property(local_id, text_property_key) == Some(value.as_slice()) {
        return Some(false);
    }
    Some(instance.set_string_property(local_id, text_property_key, value))
}

pub(crate) fn has_root_text_value_run(instance: &ArtboardInstance, name: &str) -> bool {
    root_text_value_run_local_id(instance.slots(), name).is_some()
}

fn root_text_value_run_local_id(slots: &[InstanceSlot], name: &str) -> Option<usize> {
    slots
        .iter()
        .filter(|slot| slot.type_name == Some("TextValueRun") && slot.name.as_deref() == Some(name))
        .min_by_key(|slot| slot.local_id)
        .map(|slot| slot.local_id)
}

pub(crate) fn apply_string_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    match instance.slot(local_id).and_then(|slot| slot.type_name) {
        Some("TextValueRun")
            if property_key_for_name("TextValueRun", "text") == Some(property_key) =>
        {
            mark_shape_dirty(instance, local_id)
        }
        _ => false,
    }
}

fn mark_shape_dirty(instance: &mut ArtboardInstance, run_local_id: usize) -> bool {
    let Some(parent_key) = property_key_for_name("Component", "parentId") else {
        return false;
    };
    let Some(text_local) = instance
        .uint_property(run_local_id, parent_key)
        .and_then(|parent_id| usize::try_from(parent_id).ok())
    else {
        return false;
    };
    if instance.slot(text_local).and_then(|slot| slot.type_name) != Some("Text") {
        return false;
    }

    let mut changed = false;
    changed |= instance.add_dirt(text_local, ComponentDirt::TEXT_SHAPE, false);
    changed |= instance.add_dirt(text_local, ComponentDirt::WORLD_TRANSFORM, true);
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_lookup_uses_first_exact_text_value_run_match() {
        let slots = [
            InstanceSlot {
                local_id: 7,
                source_global_id: 0,
                type_name: Some("TextValueRun"),
                name: Some("headline".to_owned()),
            },
            InstanceSlot {
                local_id: 2,
                source_global_id: 1,
                type_name: Some("TextValueRun"),
                name: Some("headline".to_owned()),
            },
            InstanceSlot {
                local_id: 1,
                source_global_id: 2,
                type_name: Some("Text"),
                name: Some("headline".to_owned()),
            },
            InstanceSlot {
                local_id: 0,
                source_global_id: 3,
                type_name: Some("TextValueRun"),
                name: Some("Headline".to_owned()),
            },
        ];

        assert_eq!(root_text_value_run_local_id(&slots, "headline"), Some(2));
        assert_eq!(root_text_value_run_local_id(&slots, "Headline"), Some(0));
        assert_eq!(root_text_value_run_local_id(&slots, "missing"), None);
    }
}
