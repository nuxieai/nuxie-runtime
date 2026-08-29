//! Mechanical translation of pinned `StateTransitionImporter`.

use super::*;

pub(super) fn dispatch_imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateTransition") {
        // `StateTransition::import` must attach the transition to the latest
        // LayerState before File constructs this importer.
        return Some(context.latest(ImportStackKey::LayerState));
    }
    if definition.is_a("TransitionCondition") {
        // `TransitionCondition::import` retrieves this retained importer and
        // calls `addCondition` before delegating to Super.
        return Some(context.latest(ImportStackKey::StateTransition));
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.is_a("StateTransition") {
        context.make_latest(ImportStackKey::StateTransition);
    }
}

/// Occurrence coordinates are the Rust equivalent of the pinned importer's
/// retained `StateTransition* m_Transition` relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StateTransitionImporter {
    state_machine_index: usize,
    layer_index: usize,
    state_index: usize,
    transition_index: usize,
}

impl StateTransitionImporter {
    /// Mechanical translation of the constructor: retain exactly the
    /// transition supplied when File creates the importer.
    pub(super) fn new(
        state_machine_index: usize,
        layer_index: usize,
        state_index: usize,
        transition_index: usize,
    ) -> Self {
        Self {
            state_machine_index,
            layer_index,
            state_index,
            transition_index,
        }
    }

    /// Mechanical translation of `addCondition`: append directly to the
    /// retained transition in callback/file order.
    pub(super) fn add_condition<'a>(
        &self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        condition: &'a RuntimeObject,
    ) {
        state_machines[self.state_machine_index].layers[self.layer_index].states[self.state_index]
            .transitions[self.transition_index]
            .conditions
            .push(condition);
    }

    /// Pinned `resolve` always returns `StatusCode::Ok`; Rust represents that
    /// infallible status as `()`.
    pub(super) fn resolve(self) {}
}
