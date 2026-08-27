use super::*;

/// Rust retains the listener-input-type occurrence as its stable owning slot
/// instead of the pinned raw `ListenerInputTypeSemantic*`.
#[derive(Debug, Clone, Copy)]
pub(super) struct ListenerInputTypeSemanticImporter {
    listener_input_type_semantic: RuntimeStateMachineListenerInputTypeOwner,
}

impl ListenerInputTypeSemanticImporter {
    /// Mechanical translation of `ListenerInputTypeSemanticImporter`'s
    /// constructor.
    pub(super) fn new(
        listener_input_type_semantic: RuntimeStateMachineListenerInputTypeOwner,
    ) -> Self {
        Self {
            listener_input_type_semantic,
        }
    }

    /// Mechanical translation of `listenerInputTypeSemantic()`.
    pub(super) fn listener_input_type_semantic(
        &self,
    ) -> RuntimeStateMachineListenerInputTypeOwner {
        self.listener_input_type_semantic
    }

    /// Mechanical translation of `resolve() -> StatusCode::Ok`.
    pub(super) fn resolve(&self) -> bool {
        true
    }
}

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
