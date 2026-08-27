use super::RuntimeLayerState;

// `StateMachineLayer::import`, `addState`, `onAddedDirty`, and
// `onAddedClean` execute while the file still owns mutable Core objects.
// `nuxie-binary` preserves that phase: it attaches each layer to the latest
// StateMachine, appends states in authored order, runs their lifecycle before
// validating Any/Entry/Exit, and records whether the owner Artboard had
// already completed initialization. This runtime owner is the immutable result.

/// Immutable authored state-machine layer definition.
///
/// Pinned C++ retains states in insertion order and resolves the last authored
/// Any/Entry/Exit state during dirty finalization.
#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer {
    pub global_id: u32,
    pub name: Option<String>,
    // C++ deletes every retained `LayerState*` in
    // `~StateMachineLayer`. Rust's owned Vec performs the same ownership
    // teardown automatically.
    pub states: Vec<RuntimeLayerState>,
    pub(crate) entry_state_index: Option<usize>,
    pub(crate) any_state_index: Option<usize>,
    pub(crate) exit_state_index: Option<usize>,
}

impl RuntimeStateMachineLayer {
    pub(crate) fn resolve_system_state_indices(
        states: &[RuntimeLayerState],
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        let mut any_state_index = None;
        let mut entry_state_index = None;
        let mut exit_state_index = None;
        for (state_index, state) in states.iter().enumerate() {
            match state.type_name {
                Some("AnyState") => any_state_index = Some(state_index),
                Some("EntryState") => entry_state_index = Some(state_index),
                Some("ExitState") => exit_state_index = Some(state_index),
                _ => {}
            }
        }
        (entry_state_index, any_state_index, exit_state_index)
    }

    pub(crate) fn any_state_index(&self) -> Option<usize> {
        self.any_state_index
    }

    pub(crate) fn entry_state_index(&self) -> Option<usize> {
        self.entry_state_index
    }

    pub(crate) fn exit_state_index(&self) -> Option<usize> {
        self.exit_state_index
    }

    pub(crate) fn state_count(&self) -> usize {
        self.states.len()
    }

    pub(crate) fn state(&self, index: usize) -> Option<&RuntimeLayerState> {
        if index < self.state_count() {
            return self.states.get(index);
        }
        None
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
    fn last_authored_system_state_wins() {
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
