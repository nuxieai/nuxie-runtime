use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "SemanticInput" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("SemanticInput is owned by ListenerInputTypeSemanticImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "ListenerInputTypeSemantic" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "SemanticInput").then(|| {
        context.latest(ImportStackKey::Artboard)
            && context.latest(ImportStackKey::ListenerInputTypeSemantic)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ListenerInputTypeSemantic" {
        context.make_latest(ImportStackKey::ListenerInputTypeSemantic);
    }
}
