use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ViewModelInstance" {
        // Publisher-era instances attach to ViewModel; current instances attach
        // to Backboard and resolve viewModelId.
        return Some(
            context.latest(ImportStackKey::Backboard) || context.latest(ImportStackKey::ViewModel),
        );
    }
    if matches!(
        definition.name,
        "ViewModelInstanceAsset" | "ViewModelInstanceAssetImage" | "ViewModelInstanceAssetFont"
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
