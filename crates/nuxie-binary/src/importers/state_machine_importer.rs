//! Mechanical translation of pinned `StateMachineImporter`.

use super::*;

pub(super) fn dispatch_imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("StateMachineInput")
        .then(|| context.latest(ImportStackKey::StateMachine))
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "StateMachine").then(|| context.latest(ImportStackKey::Artboard))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "StateMachine" {
        context.state_machine_inputs.clear();
        context.make_latest(ImportStackKey::StateMachine);
    }
}

pub(super) fn dispatch_update_input_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if let Some(kind) = state_machine_input_kind(definition) {
        context.state_machine_inputs.push(Some(kind));
    }
}

pub(super) fn read_null_object_context(
    inputs: &mut Vec<Option<StateMachineInputKind>>,
) -> bool {
    read_null_object_into(inputs)
}

fn read_null_object_into<T>(inputs: &mut Vec<Option<T>>) -> bool {
    inputs.push(None);
    true
}

/// The index is the Rust equivalent of pinned `StateMachine* m_StateMachine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StateMachineImporter {
    state_machine_index: usize,
}

impl StateMachineImporter {
    pub(super) fn new(state_machine_index: usize) -> Self {
        Self {
            state_machine_index,
        }
    }

    /// Mechanical translation of the primary-header `stateMachine()` inline.
    pub(super) fn state_machine(self) -> usize {
        self.state_machine_index
    }

    pub(super) fn add_layer<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        layer: &'a RuntimeObject,
        lifecycle_applied: bool,
    ) -> usize {
        let machine = &mut state_machines[self.state_machine_index];
        machine.layers.push(RuntimeStateMachineLayer {
            object: layer,
            lifecycle_applied,
            state_count: 0,
            states: Vec::new(),
        });
        machine.layers.len() - 1
    }

    pub(super) fn add_input<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        input: &'a RuntimeObject,
    ) {
        state_machines[self.state_machine_index]
            .inputs
            .push(Some(input));
    }

    pub(super) fn add_listener<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        listener: &'a RuntimeObject,
    ) -> usize {
        let machine = &mut state_machines[self.state_machine_index];
        machine.listeners.push(RuntimeStateMachineListener {
            object: listener,
            actions: Vec::new(),
            listener_input_types: Vec::new(),
            listener_input_type_inputs: Vec::new(),
        });
        machine.listeners.len() - 1
    }

    pub(super) fn add_data_bind<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        data_bind: &'a RuntimeObject,
    ) {
        state_machines[self.state_machine_index]
            .data_binds
            .push(data_bind);
    }

    pub(super) fn read_null_object<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
    ) -> bool {
        read_null_object_into(&mut state_machines[self.state_machine_index].inputs)
    }

    pub(super) fn add_scripted_object<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        object: &'a RuntimeObject,
    ) -> usize {
        let machine = &mut state_machines[self.state_machine_index];
        machine.scripted_objects.push(RuntimeScriptedObject {
            object,
            inputs: Vec::new(),
        });
        machine.scripted_objects.len() - 1
    }

    /// Pinned `resolve` always returns `StatusCode::Ok`.
    pub(super) fn resolve(self) {}
}
