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
    listener_importer: state_machine_listener_importer::StateMachineListenerImporter,
    input_type: &'a RuntimeObject,
) -> RuntimeStateMachineListenerInputTypeOwner {
    listener_importer.add_listener_input_type(state_machines, input_type)
}
