use super::*;

fn on_added_clean(parent: &RuntimeObject) -> bool {
    definition_by_type_key(parent.type_key).is_some_and(|definition| definition.name == "DashPath")
}

pub(super) fn validate_cpp_parentage(
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> Result<()> {
    for slot in slots {
        let Some(file_index) = *slot else {
            continue;
        };
        let Some(object) = objects[file_index].as_ref() else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };
        if definition.name != "Dash" {
            continue;
        }

        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            continue;
        };
        if !on_added_clean(parent) {
            bail!("dash object {} has parent that is not DashPath", object.id);
        }
    }

    Ok(())
}
