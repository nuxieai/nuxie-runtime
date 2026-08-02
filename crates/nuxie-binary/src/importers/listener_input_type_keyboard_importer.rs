use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "KeyboardInput" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("KeyboardInput is owned by ListenerInputTypeKeyboardImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "ListenerInputTypeKeyboard" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "KeyboardInput").then(|| {
        context.latest(ImportStackKey::Artboard)
            && context.latest(ImportStackKey::ListenerInputTypeKeyboard)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ListenerInputTypeKeyboard" {
        context.make_latest(ImportStackKey::ListenerInputTypeKeyboard);
    }
}
