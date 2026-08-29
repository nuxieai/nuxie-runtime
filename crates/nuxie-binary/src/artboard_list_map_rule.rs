//! Direct import owner for pinned C++ `src/artboard_list_map_rule.cpp`.

use super::*;

/// After the shared Component import path has applied `Super::onAddedDirty`,
/// `ArtboardListMapRule::onAddedDirty` accepts the rule only when its resolved
/// Component parent is an `ArtboardComponentList`.
pub(super) fn on_added_dirty_parent_is_valid(
    object: &RuntimeObject,
    artboard_local_slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> bool {
    let Some(parent) = local_object_reference(
        artboard_local_slots,
        objects,
        object.uint_property("parentId"),
    ) else {
        return false;
    };
    runtime_object_is_cpp_artboard_component_list(parent)
}

impl RuntimeFile {
    pub fn artboard_component_list_map_rules(
        &self,
        list_id: usize,
    ) -> Vec<RuntimeArtboardListMapRule<'_>> {
        let Some(list) = self.object(list_id) else {
            return Vec::new();
        };

        self.artboard_component_list_map_rules_for_object(list)
    }

    /// Imported rule objects accepted beneath this component-list parent, in
    /// serialized order before C++ `addMapRule` overwrite semantics apply.
    pub fn artboard_component_list_map_rules_for_object(
        &self,
        list: &RuntimeObject,
    ) -> Vec<RuntimeArtboardListMapRule<'_>> {
        if !runtime_object_is_cpp_artboard_component_list(list) {
            return Vec::new();
        }

        let Some((_, range, slots, list_local_index)) =
            self.cpp_artboard_local_context_for_object(list)
        else {
            return Vec::new();
        };

        self.objects[range.0..range.1]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let file_index = range.0 + offset;
                if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                if object.type_name != "ArtboardListMapRule" {
                    return None;
                }
                if !slots.iter().any(|slot| *slot == Some(file_index)) {
                    return None;
                }
                if object.uint_property("parentId") != Some(list_local_index as u64) {
                    return None;
                }

                Some(RuntimeArtboardListMapRule {
                    object,
                    view_model_id: object.uint_property("viewModelId")?,
                    artboard_id: object.uint_property("artboardId")?,
                })
            })
            .collect()
    }

    /// The table produced by `ArtboardListMapRule::onAddedDirty` calling
    /// `ArtboardComponentList::addMapRule`: later rules replace earlier rules
    /// with the same unsigned `viewModelId`.
    pub fn registered_artboard_component_list_map_rules_for_object(
        &self,
        list: &RuntimeObject,
    ) -> Vec<RuntimeArtboardListMapRule<'_>> {
        let mut map_rules = BTreeMap::new();
        for rule in self.artboard_component_list_map_rules_for_object(list) {
            map_rules.insert(rule.view_model_id, rule);
        }
        map_rules.into_values().collect()
    }
}
