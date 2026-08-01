use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "Backboard" {
        return Some(true);
    }
    if definition.is_a("DataBind") || definition.is_a("DataConverter") {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    if definition.is_a("ScrollPhysics") {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    definition.is_a("KeyFrameInterpolator").then(|| {
        context.latest(ImportStackKey::Artboard) || context.latest(ImportStackKey::Backboard)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "Backboard" {
        context.make_latest(ImportStackKey::Backboard);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_data_converters(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                definition_by_type_key(object.type_key)
                    .is_some_and(|definition| definition.is_a("DataConverter"))
                    .then_some(object)
            })
    }
}
