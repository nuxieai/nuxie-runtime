use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ScriptInputArtboard" {
        return Some(
            context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::ScriptedObject),
        );
    }
    definition
        .name
        .starts_with("ScriptInput")
        .then(|| context.latest(ImportStackKey::ScriptedObject))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition_is_cpp_scripted_object(definition) {
        context.make_latest(ImportStackKey::ScriptedObject);
    }
}
impl RuntimeFile {
    /// Script inputs attached by the pinned `ScriptedObjectImporter`.
    ///
    /// ScriptInput records do not carry an owner id. The importer assigns
    /// them to the latest successfully imported ScriptedObject, so reproduce
    /// that authored-order ownership directly instead of guessing from
    /// Component parent links.
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
