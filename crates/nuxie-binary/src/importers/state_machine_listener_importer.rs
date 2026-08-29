use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateMachineListener") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("listener is owned by StateMachineListenerImporter"),
        );
    }
    if definition.is_a("ListenerAction") && listener_action_parent_kind_is_listener(object) {
        return Some(
            imports_successfully(object, definition, context)
                .expect("ListenerAction has a concrete importer owner"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.is_a("StateMachineListener") {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateMachineListener") {
        return Some(context.latest(ImportStackKey::StateMachine));
    }
    (definition.is_a("ListenerAction") && listener_action_parent_kind_is_listener(object))
        .then(|| listener_action_imports_successfully(object, context))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("StateMachineListener") {
        context.make_latest(ImportStackKey::StateMachineListener);
    }
}

/// Stable coordinates are the Rust counterpart of the retained
/// `StateMachineListener*` in the pinned importer.
#[derive(Debug, Clone, Copy)]
pub(super) struct StateMachineListenerImporter {
    state_machine_listener: RuntimeStateMachineListenerOwner,
}

impl StateMachineListenerImporter {
    pub(super) fn new(state_machine_listener: RuntimeStateMachineListenerOwner) -> Self {
        Self {
            state_machine_listener,
        }
    }

    /// Mechanical translation of the primary-header
    /// `stateMachineListener()` inline.
    pub(super) fn state_machine_listener(self) -> RuntimeStateMachineListenerOwner {
        self.state_machine_listener
    }

    pub(super) fn add_action<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        action: RuntimeListenerAction<'a>,
    ) {
        state_machines[self.state_machine_listener.state_machine_index].listeners
            [self.state_machine_listener.listener_index]
            .actions
            .push(action);
    }

    pub(super) fn add_listener_input_type<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        input_type: &'a RuntimeObject,
    ) -> RuntimeStateMachineListenerInputTypeOwner {
        let listener = &mut state_machines[self.state_machine_listener.state_machine_index]
            .listeners[self.state_machine_listener.listener_index];
        listener.listener_input_types.push(input_type);
        listener.listener_input_type_inputs.push(Vec::new());

        RuntimeStateMachineListenerInputTypeOwner {
            state_machine_index: self.state_machine_listener.state_machine_index,
            listener_index: self.state_machine_listener.listener_index,
            input_type_index: listener.listener_input_types.len() - 1,
        }
    }

    /// Pinned `resolve` always returns `StatusCode::Ok`.
    pub(super) fn resolve(self) {}
}
