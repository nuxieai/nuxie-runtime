use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "TextAsset" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("TextAsset is owned by TextAssetImporter"),
        );
    }
    None
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "TextAsset").then(|| context.latest(ImportStackKey::Backboard))
}
