use super::*;

/// Rust retains the listener-input-type occurrence as its stable owning slot
/// instead of the pinned raw `ListenerInputTypeGamepad*`.
#[derive(Debug, Clone, Copy)]
pub(super) struct ListenerInputTypeGamepadImporter {
    listener_input_type_gamepad: RuntimeStateMachineListenerInputTypeOwner,
}

impl ListenerInputTypeGamepadImporter {
    /// Mechanical translation of `ListenerInputTypeGamepadImporter`'s
    /// constructor.
    pub(super) fn new(
        listener_input_type_gamepad: RuntimeStateMachineListenerInputTypeOwner,
    ) -> Self {
        Self {
            listener_input_type_gamepad,
        }
    }

    /// Mechanical translation of `listenerInputTypeGamepad()`.
    pub(super) fn listener_input_type_gamepad(
        &self,
    ) -> RuntimeStateMachineListenerInputTypeOwner {
        self.listener_input_type_gamepad
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
    if definition.name == "GamepadInput" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("GamepadInput is owned by ListenerInputTypeGamepadImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "ListenerInputTypeGamepad" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "GamepadInput").then(|| {
        context.latest(ImportStackKey::Artboard)
            && context.latest(ImportStackKey::ListenerInputTypeGamepad)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ListenerInputTypeGamepad" {
        context.make_latest(ImportStackKey::ListenerInputTypeGamepad);
    }
}
