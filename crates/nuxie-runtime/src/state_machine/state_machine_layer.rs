use super::RuntimeLayerState;
use anyhow::{Result, ensure};

/// Immutable authored state-machine layer definition.
///
/// Pinned C++ retains states in insertion order and resolves the last authored
/// Any/Entry/Exit state during dirty finalization.
#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer {
    pub global_id: u32,
    pub name: Option<String>,
    pub states: Vec<RuntimeLayerState>,
    pub(crate) entry_state_index: Option<usize>,
    pub(crate) any_state_index: Option<usize>,
    pub(crate) exit_state_index: Option<usize>,
}

impl RuntimeStateMachineLayer {
    pub(crate) fn resolve_system_state_indices(
        states: &[RuntimeLayerState],
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        (
            states
                .iter()
                .rposition(|state| state.type_name == Some("EntryState")),
            states
                .iter()
                .rposition(|state| state.type_name == Some("AnyState")),
            states
                .iter()
                .rposition(|state| state.type_name == Some("ExitState")),
        )
    }

    /// Validate the references finalized by pinned C++
    /// `StateMachineLayer::onAddedDirty` and
    /// `StateMachineLayerImporter::resolve`.
    ///
    /// Keeping this on the focused layer owner prevents malformed layers from
    /// becoming partially live Rust definitions whose missing transitions are
    /// silently skipped during advance.
    pub(crate) fn validate_imported_references(&self) -> Result<()> {
        ensure!(
            self.any_state_index.is_some(),
            "state-machine layer {} is missing AnyState",
            self.global_id
        );
        ensure!(
            self.entry_state_index.is_some(),
            "state-machine layer {} is missing EntryState",
            self.global_id
        );
        ensure!(
            self.exit_state_index.is_some(),
            "state-machine layer {} is missing ExitState",
            self.global_id
        );
        for (state_index, state) in self.states.iter().enumerate() {
            for (transition_index, transition) in state.transitions.iter().enumerate() {
                ensure!(
                    transition
                        .state_to_index
                        .is_some_and(|target| target < self.states.len()),
                    "state-machine layer {} state {} transition {} has an invalid target",
                    self.global_id,
                    state_index,
                    transition_index
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(type_name: Option<&'static str>) -> RuntimeLayerState {
        RuntimeLayerState {
            global_id: None,
            type_name,
            animation: None,
            blend_state_1d: None,
            blend_state_direct: None,
            speed: 1.0,
            flags: 0,
            fire_actions: Vec::new(),
            listener_actions: Vec::new(),
            transitions: Vec::new(),
        }
    }

    #[test]
    fn required_system_states_are_independent_and_last_authored_wins() {
        for missing in ["AnyState", "EntryState", "ExitState"] {
            let states = ["AnyState", "EntryState", "ExitState"]
                .into_iter()
                .filter(|type_name| *type_name != missing)
                .map(|type_name| state(Some(type_name)))
                .collect::<Vec<_>>();
            let (entry_state_index, any_state_index, exit_state_index) =
                RuntimeStateMachineLayer::resolve_system_state_indices(&states);
            let layer = RuntimeStateMachineLayer {
                global_id: 17,
                name: None,
                states,
                entry_state_index,
                any_state_index,
                exit_state_index,
            };
            assert!(
                layer.validate_imported_references().is_err(),
                "{missing} must be required independently"
            );
        }

        let states = vec![
            state(Some("AnyState")),
            state(Some("EntryState")),
            state(Some("ExitState")),
            state(None),
            state(Some("AnyState")),
            state(Some("EntryState")),
            state(Some("ExitState")),
        ];
        assert_eq!(
            RuntimeStateMachineLayer::resolve_system_state_indices(&states),
            (Some(5), Some(4), Some(6))
        );
    }
}
