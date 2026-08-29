use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "KeyedObject" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("KeyedObject is owned by KeyedObjectImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "KeyedObject" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "KeyedObject").then(|| context.latest(ImportStackKey::LinearAnimation))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "KeyedObject" {
        context.make_latest(ImportStackKey::KeyedObject);
    }
}
