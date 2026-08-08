use super::*;

/// Immutable authored state definition.
///
/// Mirrors pinned C++ `LayerState`: transitions and scheduled actions belong
/// to the definition, while mutable animation/blend state belongs to a
/// separate `RuntimeStateInstance` occurrence.
#[derive(Debug, Clone)]
pub struct RuntimeLayerState {
    pub global_id: Option<u32>,
    pub type_name: Option<&'static str>,
    pub(crate) animation: Option<RuntimeLinearAnimationHandle>,
    pub(crate) blend_state_1d: Option<RuntimeBlendState1D>,
    pub(crate) blend_state_direct: Option<RuntimeBlendStateDirect>,
    pub(crate) speed: f32,
    pub(crate) flags: u64,
    pub(crate) fire_actions: Vec<RuntimeStateMachineFireAction>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
    pub(crate) transitions: Vec<RuntimeStateTransition>,
}

impl RuntimeLayerState {
    const RANDOM: u64 = 1 << 0;
    const RESET: u64 = 1 << 1;

    /// Authored Core object id of this state, when the binary supplies one.
    pub fn global_id(&self) -> Option<u32> {
        self.global_id
    }

    /// Pinned Core schema type key for this authored state. A state without a
    /// more-derived record is the base `LayerState`, matching C++ `typeKey`.
    pub fn core_type(&self) -> Option<u32> {
        nuxie_schema::definition_by_name(self.type_name.unwrap_or("LayerState"))
            .map(|definition| u32::from(definition.type_key.int))
    }

    pub(super) fn uses_random_transition_selection(&self) -> bool {
        self.flags & Self::RANDOM == Self::RANDOM
    }

    pub(super) fn resets_blend_values(&self) -> bool {
        self.flags & Self::RESET == Self::RESET
    }

    pub(super) fn perform_fire_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        artboard: &ArtboardInstance,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        perform_state_machine_fire_actions(
            &self.fire_actions,
            occurrence,
            artboard,
            executor,
            reported_events,
        );
    }

    pub(super) fn perform_listener_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        artboard: &mut ArtboardInstance,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<bool, ScriptError> {
        perform_scheduled_listener_actions(
            &self.listener_actions,
            occurrence,
            artboard,
            targets,
            executor,
        )
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
    fn core_type_distinguishes_base_and_derived_layer_states() {
        let base = state(None).core_type().expect("LayerState schema key");
        let animation = state(Some("AnimationState"))
            .core_type()
            .expect("AnimationState schema key");
        let exit = state(Some("ExitState"))
            .core_type()
            .expect("ExitState schema key");
        assert_ne!(base, animation);
        assert_ne!(animation, exit);
        assert_eq!(
            base,
            u32::from(
                nuxie_schema::definition_by_name("LayerState")
                    .expect("LayerState definition")
                    .type_key
                    .int
            )
        );
    }
}
