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

    pub(super) fn uses_random_transition_selection(&self) -> bool {
        self.flags & Self::RANDOM == Self::RANDOM
    }

    pub(super) fn resets_blend_values(&self) -> bool {
        self.flags & Self::RESET == Self::RESET
    }

    pub(super) fn perform_fire_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        perform_state_machine_fire_actions(
            &self.fire_actions,
            occurrence,
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
