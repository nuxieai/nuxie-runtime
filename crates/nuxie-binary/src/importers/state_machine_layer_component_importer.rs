use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateMachineFireAction") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("fire action is owned by StateMachineLayerComponentImporter"),
        );
    }
    if definition.is_a("ListenerAction") && !listener_action_parent_kind_is_listener(object) {
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
    if definition.is_a("StateMachineLayerComponent") {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateMachineFireAction") {
        return Some(context.latest(ImportStackKey::StateMachineLayerComponent));
    }
    (definition.is_a("ListenerAction") && !listener_action_parent_kind_is_listener(object))
        .then(|| listener_action_imports_successfully(object, context))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("StateMachineLayerComponent") {
        context.make_latest(ImportStackKey::StateMachineLayerComponent);
    }
}

/// Occurrence coordinates are the Rust equivalent of the pinned importer's
/// retained `StateMachineLayerComponent* m_stateMachineLayerComponent`.
#[derive(Debug, Clone, Copy)]
pub(super) struct StateMachineLayerComponentImporter {
    owner: RuntimeStateMachineLayerComponentOwner,
}

impl StateMachineLayerComponentImporter {
    /// Mechanical translation of the constructor: retain exactly the
    /// component supplied when File creates the importer.
    pub(super) fn new(owner: RuntimeStateMachineLayerComponentOwner) -> Self {
        Self { owner }
    }

    pub(super) fn state_machine_index(self) -> usize {
        match self.owner {
            RuntimeStateMachineLayerComponentOwner::State {
                state_machine_index,
                ..
            }
            | RuntimeStateMachineLayerComponentOwner::Transition {
                state_machine_index,
                ..
            } => state_machine_index,
        }
    }

    /// Mechanical translation of `addFireEvent`: append the raw occurrence
    /// directly to the retained component in callback/file order.
    pub(super) fn add_fire_event<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        object: &'a RuntimeObject,
        artboard_local_slots: &[Option<usize>],
        objects: &'a [Option<RuntimeObject>],
    ) {
        let (event_local_index, event) =
            cpp_resolved_action_event(object, artboard_local_slots, objects);
        let fire_event = RuntimeStateMachineFireAction {
            object,
            event_local_index,
            event,
        };

        match self.owner {
            RuntimeStateMachineLayerComponentOwner::State {
                state_machine_index,
                layer_index,
                state_index,
            } => state_machines[state_machine_index].layers[layer_index].states[state_index]
                .fire_actions
                .push(fire_event),
            RuntimeStateMachineLayerComponentOwner::Transition {
                state_machine_index,
                layer_index,
                state_index,
                transition_index,
            } => state_machines[state_machine_index].layers[layer_index].states[state_index]
                .transitions[transition_index]
                .fire_actions
                .push(fire_event),
        }
    }

    /// Mechanical translation of `addListenerAction`: transfer the owned
    /// action into the retained component in callback/file order. Rust's
    /// vectors provide the pinned component destructor's recursive cleanup.
    pub(super) fn add_listener_action<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        action: RuntimeListenerAction<'a>,
    ) {
        match self.owner {
            RuntimeStateMachineLayerComponentOwner::State {
                state_machine_index,
                layer_index,
                state_index,
            } => state_machines[state_machine_index].layers[layer_index].states[state_index]
                .listener_actions
                .push(action),
            RuntimeStateMachineLayerComponentOwner::Transition {
                state_machine_index,
                layer_index,
                state_index,
                transition_index,
            } => state_machines[state_machine_index].layers[layer_index].states[state_index]
                .transitions[transition_index]
                .listener_actions
                .push(action),
        }
    }
}
