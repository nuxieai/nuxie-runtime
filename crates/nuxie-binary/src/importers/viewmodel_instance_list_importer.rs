use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "ViewModelInstanceListItem")
        .then(|| context.latest(ImportStackKey::ViewModelInstanceList))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ViewModelInstanceList" {
        context.make_latest(ImportStackKey::ViewModelInstanceList);
    }
}
