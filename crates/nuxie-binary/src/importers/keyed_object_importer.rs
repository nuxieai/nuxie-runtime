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
    if definition.is_a("KeyFrame") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("KeyFrame is owned through KeyedObjectImporter"),
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
    if definition.name == "KeyedObject" {
        return Some(context.latest(ImportStackKey::LinearAnimation));
    }
    definition
        .is_a("KeyFrame")
        .then(|| context.latest(ImportStackKey::KeyedProperty))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "KeyedObject" {
        context.make_latest(ImportStackKey::KeyedObject);
    }
}
