use super::*;

/// First-pass half of `ListenerInputType::import`: the C++ import stack must
/// contain a `StateMachineListenerImporter` before the occurrence can import.
pub(super) fn imports_successfully(
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("ListenerInputType")
        .then(|| context.latest(ImportStackKey::StateMachineListener))
}

/// Retained-graph half of `ListenerInputType::import`: transfer this concrete
/// occurrence to the latest listener, preserving insertion order, then return
/// the stable Rust owner used by concrete input-type importers.
///
/// Pinned `Core::import` is an unconditional `StatusCode::Ok`, so the Rust
/// two-pass import adaptation has no later fallible superclass action to run.
pub(super) fn import<'a>(
    state_machines: &mut [RuntimeStateMachine<'a>],
    listener_owner: RuntimeStateMachineListenerOwner,
    input_type: &'a RuntimeObject,
) -> RuntimeStateMachineListenerInputTypeOwner {
    let listener = &mut state_machines[listener_owner.state_machine_index].listeners
        [listener_owner.listener_index];
    listener.listener_input_types.push(input_type);
    listener.listener_input_type_inputs.push(Vec::new());

    RuntimeStateMachineListenerInputTypeOwner {
        state_machine_index: listener_owner.state_machine_index,
        listener_index: listener_owner.listener_index,
        input_type_index: listener.listener_input_types.len() - 1,
    }
}
