use super::*;

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
