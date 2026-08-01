use super::*;

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
