use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ViewModelInstance" {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    if matches!(
        definition.name,
        "ViewModelInstanceAsset"
            | "ViewModelInstanceAssetImage"
            | "ViewModelInstanceAssetFont"
            | "ViewModelInstanceAssetBlob"
    ) {
        return Some(
            context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::ViewModelInstance),
        );
    }
    definition
        .is_a("ViewModelInstanceValue")
        .then(|| context.latest(ImportStackKey::ViewModelInstance))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ViewModelInstance" {
        context.make_latest(ImportStackKey::ViewModelInstance);
    }
}
