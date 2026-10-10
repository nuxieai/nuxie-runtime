use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ScriptInputArtboard" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("ScriptInputArtboard is owned by ScriptedObjectImporter"),
        );
    }
    if definition.name.starts_with("ScriptInput") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("ScriptInput is owned by ScriptedObjectImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition_is_cpp_scripted_object(definition) {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition.name.starts_with("ScriptInput").then(|| {
        context.latest(ImportStackKey::ScriptedObject)
            && context.scripted_input_parent_id
                == Some(object.uint_property("parentId").unwrap_or(0))
            && (definition.name != "ScriptInputArtboard"
                || context.latest(ImportStackKey::Backboard))
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition_is_cpp_scripted_object(definition) {
        context.scripted_input_parent_id = Some(
            if definition.is_a("Component") && context.latest(ImportStackKey::Artboard) {
                (context.artboard_local_nested_inputs.len() - 1) as u64
            } else {
                0
            },
        );
        context.make_latest(ImportStackKey::ScriptedObject);
    }
}
impl RuntimeFile {
    /// Script inputs attached by the pinned `ScriptedObjectImporter`.
    ///
    /// The latest ScriptedObject owns only inputs whose serialized parent
    /// matches its artboard slot (or zero for a non-Component owner).
    pub fn scripted_inputs_for_object<'a>(
        &'a self,
        scripted_object: &RuntimeObject,
    ) -> Vec<&'a RuntimeObject> {
        let Some(owner_id) = usize::try_from(scripted_object.id).ok() else {
            return Vec::new();
        };
        if self.import_status(owner_id) != Some(RuntimeImportStatus::Imported)
            || !definition_by_type_key(scripted_object.type_key)
                .is_some_and(definition_is_cpp_scripted_object)
        {
            return Vec::new();
        }

        let mut inputs = Vec::new();
        for candidate in self
            .objects
            .iter()
            .skip(owner_id.saturating_add(1))
            .flatten()
        {
            let Some(candidate_id) = usize::try_from(candidate.id).ok() else {
                continue;
            };
            if self.import_status(candidate_id) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(candidate.type_key) else {
                continue;
            };
            if definition_is_cpp_scripted_object(definition) {
                break;
            }
            if definition.name.starts_with("ScriptInput") {
                inputs.push(candidate);
            }
        }
        inputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(id: u32, name: &'static str, parent: u64) -> Option<RuntimeObject> {
        let definition = nuxie_schema::definition_by_name(name).unwrap();
        Some(RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties: vec![RuntimeProperty {
                key: 5,
                name: "parentId",
                owner: "Component",
                value: FieldValue::Uint(parent),
            }],
            skipped_properties: Vec::new(),
        })
    }

    // Translated from artboard_slot_import_test.cpp: unreadable owners must
    // neither steal the previous owner's inputs nor shift subsequent ids.
    #[test]
    fn unreadable_script_owner_preserves_parent_numbered_slots() {
        for previous_owner in [false, true] {
            for component_owner in [false, true] {
                let mut objects = vec![object(0, "Backboard", 0), object(1, "Artboard", 0)];
                if previous_owner {
                    objects.push(object(2, "ScriptedDrawable", 0));
                }
                objects.push(None);
                let input_index = objects.len();
                let parent = if component_owner {
                    input_index as u64 - 2
                } else {
                    0
                };
                objects.push(object(input_index as u32, "ScriptInputNumber", parent));
                let node_index = objects.len();
                objects.push(object(node_index as u32, "Node", 0));
                let statuses = compute_import_statuses(&objects, true);
                assert!(matches!(
                    statuses[input_index],
                    RuntimeImportStatus::Dropped { .. }
                ));
                let slots = runtime_artboard_local_slots(&objects, &statuses, (1, objects.len()));
                let mut expected = vec![Some(1)];
                if previous_owner {
                    expected.push(Some(2));
                }
                expected.push(None);
                if component_owner {
                    expected.push(None);
                }
                expected.push(Some(node_index));
                assert_eq!(slots, expected);
            }
        }
    }

    #[test]
    fn scripted_transition_replaces_previous_script_owner() {
        let objects = vec![
            object(0, "Backboard", 0),
            object(1, "Artboard", 0),
            object(2, "ScriptedDrawable", 0),
            object(3, "ScriptedTransition", 0),
            object(4, "ScriptInputNumber", 2),
            object(5, "ScriptInputNumber", 1),
        ];
        let statuses = compute_import_statuses(&objects, true);
        assert_eq!(statuses[3], RuntimeImportStatus::Imported);
        assert_eq!(statuses[4], RuntimeImportStatus::Imported);
        assert!(matches!(statuses[5], RuntimeImportStatus::Dropped { .. }));
        assert_eq!(
            runtime_artboard_local_slots(&objects, &statuses, (1, 6)),
            vec![Some(1), Some(2), Some(3), Some(4), None]
        );
    }

    #[test]
    fn interpolator_input_has_no_slot_and_failed_keyboard_keeps_one() {
        let objects = vec![
            object(0, "Backboard", 0),
            object(1, "Artboard", 0),
            object(2, "ScriptedInterpolator", 0),
            object(3, "ScriptInputNumber", 0),
            object(4, "KeyboardInput", 0),
            object(5, "Node", 0),
        ];
        let statuses = compute_import_statuses(&objects, true);
        assert_eq!(statuses[3], RuntimeImportStatus::Imported);
        assert!(matches!(statuses[4], RuntimeImportStatus::Dropped { .. }));
        assert_eq!(
            runtime_artboard_local_slots(&objects, &statuses, (1, 6)),
            vec![Some(1), Some(2), None, Some(5)]
        );
    }
}
