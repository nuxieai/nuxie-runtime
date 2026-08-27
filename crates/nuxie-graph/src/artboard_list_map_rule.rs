//! Graph projection for pinned C++ `src/artboard_list_map_rule.cpp`.

use super::*;

pub(super) fn component_list_map_rules(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    component_list: &RuntimeObject,
) -> Vec<ComponentListMapRuleNode> {
    let mut map_rules = BTreeMap::new();
    for rule in file.registered_artboard_component_list_map_rules_for_object(component_list) {
        let state_machine_ids = local_objects
            .iter()
            .find(|local| local.global_id == rule.object.id)
            .map(|rule_local| {
                local_objects
                    .iter()
                    .filter_map(|local| {
                        let object = runtime_object_for_local(file, local_objects, local.local_id)?;
                        (object.type_name == "NestedStateMachine"
                            && object.uint_property("parentId") == Some(rule_local.local_id as u64))
                        .then(|| {
                            object
                                .uint_property("animationId")
                                .and_then(|id| usize::try_from(id).ok())
                        })
                        .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        map_rules.insert(
            cpp_i32_from_runtime_uint(rule.view_model_id),
            (
                cpp_i32_from_runtime_uint(rule.artboard_id),
                state_machine_ids,
            ),
        );
    }

    // Preserve the graph's deterministic signed-id projection; duplicate
    // overwrite semantics have already been applied by the import owner.
    map_rules
        .into_iter()
        .map(
            |(view_model_id, (artboard_id, state_machine_ids))| ComponentListMapRuleNode {
                view_model_id,
                artboard_id,
                state_machine_ids,
            },
        )
        .collect()
}
