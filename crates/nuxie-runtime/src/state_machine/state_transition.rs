use super::{
    RuntimeScheduledListenerAction, RuntimeScheduledListenerActionExecutor,
    RuntimeScheduledListenerActionTargetsMut, RuntimeStateMachineFireAction,
    RuntimeTransitionCondition, RuntimeTransitionInterpolator, StateMachineFireOccurrence,
    StateMachineInputInstance, StateMachineReportedEvent, StateMachineTransitionDurationInstance,
    TransitionEvaluationContext, perform_scheduled_listener_actions,
    perform_state_machine_fire_actions,
};
use crate::ArtboardInstance;
use crate::animation::{AnimationLoop, LinearAnimationInstance, RuntimeLinearAnimation};
use crate::scripting::ScriptError;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateTransition {
    pub(crate) global_id: u32,
    pub(crate) state_to_index: Option<usize>,
    pub(crate) exit_blend_animation_index: Option<usize>,
    pub(crate) duration: u64,
    pub(crate) exit_time: u64,
    pub(crate) flags: u64,
    pub(crate) random_weight: u64,
    pub(crate) condition_count: usize,
    pub(super) conditions: Vec<RuntimeTransitionCondition>,
    pub(super) direct_input_conditions_only: bool,
    pub(crate) fire_actions: Vec<RuntimeStateMachineFireAction>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
    pub(crate) interpolator: Option<RuntimeTransitionInterpolator>,
    pub(crate) has_unsupported_interpolator: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionAnimationRef<'a> {
    pub(super) instance: &'a LinearAnimationInstance,
    pub(super) animation: &'a RuntimeLinearAnimation,
}

impl RuntimeStateTransition {
    const DISABLED: u64 = 1 << 0;
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
    const PAUSE_ON_EXIT: u64 = 1 << 4;
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    pub(super) fn is_simple_supported(&self) -> bool {
        self.state_to_index.is_some()
            && self.condition_count == self.conditions.len()
            && !self.has_unsupported_interpolator
            && self.flags & Self::DISABLED == 0
    }

    pub(super) fn allow(
        &self,
        context: &TransitionEvaluationContext<'_>,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        executor: &dyn RuntimeScheduledListenerActionExecutor,
        animation_from: Option<RuntimeTransitionAnimationRef<'_>>,
    ) -> TransitionAllowance {
        for condition in &self.conditions {
            if !condition.evaluate(context, artboard, inputs, executor) {
                return TransitionAllowance::No;
            }
        }

        self.allow_exit_time(animation_from)
    }

    pub(super) fn allow_direct_inputs(
        &self,
        context: &TransitionEvaluationContext<'_>,
        inputs: &[StateMachineInputInstance],
        animation_from: Option<RuntimeTransitionAnimationRef<'_>>,
    ) -> TransitionAllowance {
        debug_assert!(self.direct_input_conditions_only);
        for condition in &self.conditions {
            if !condition
                .evaluate_direct_input(inputs, context.layer_index)
                .unwrap_or(false)
            {
                return TransitionAllowance::No;
            }
        }
        self.allow_exit_time(animation_from)
    }

    fn allow_exit_time(
        &self,
        animation_from: Option<RuntimeTransitionAnimationRef<'_>>,
    ) -> TransitionAllowance {
        if self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
            && let Some(animation_from) = animation_from
        {
            let mut exit_time = self.exit_time_seconds(Some(animation_from.animation), false);
            let duration = animation_from.animation.duration_seconds();
            if exit_time <= duration
                && AnimationLoop::from_loop_value(animation_from.animation.loop_value as i32)
                    != AnimationLoop::OneShot
                && duration != 0.0
            {
                exit_time +=
                    (animation_from.instance.last_total_time / duration).floor() * duration;
            }
            if animation_from.instance.total_time < exit_time {
                return TransitionAllowance::WaitingForExit;
            }
        }

        TransitionAllowance::Yes
    }

    pub(super) fn has_exit_time(&self) -> bool {
        self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
    }

    pub(super) fn pause_on_exit(&self) -> bool {
        self.flags & Self::PAUSE_ON_EXIT == Self::PAUSE_ON_EXIT
    }

    pub(super) fn enable_early_exit(&self) -> bool {
        self.flags & Self::ENABLE_EARLY_EXIT == Self::ENABLE_EARLY_EXIT
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

    pub(super) fn transition_duration_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        duration_override: Option<f32>,
    ) -> f32 {
        let duration = duration_override
            .map(|value| if value < 0.0 { 0 } else { value.round() as u64 })
            .unwrap_or(self.duration);
        if duration == 0 {
            return 0.0;
        }
        if self.flags & Self::DURATION_IS_PERCENTAGE == Self::DURATION_IS_PERCENTAGE {
            return animation_from
                .map(|animation| duration as f32 / 100.0 * animation.duration_seconds())
                .unwrap_or(0.0);
        }
        duration as f32 / 1000.0
    }

    pub(super) fn exit_time_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        absolute: bool,
    ) -> f32 {
        if self.flags & Self::EXIT_TIME_IS_PERCENTAGE == Self::EXIT_TIME_IS_PERCENTAGE {
            return animation_from
                .map(|animation| {
                    let start = if absolute {
                        animation.start_seconds()
                    } else {
                        0.0
                    };
                    start + self.exit_time as f32 / 100.0 * animation.duration_seconds()
                })
                .unwrap_or(0.0);
        }
        self.exit_time as f32 / 1000.0
    }

    pub(super) fn use_inputs(
        &self,
        inputs: &mut [StateMachineInputInstance],
        executor: &dyn RuntimeScheduledListenerActionExecutor,
        layer_index: usize,
        view_model_trigger_layer_id: u64,
    ) {
        for condition in &self.conditions {
            condition.use_input(executor, inputs, layer_index, view_model_trigger_layer_id);
        }
    }
}

pub(super) fn transition_duration_value(
    transition_durations: &[StateMachineTransitionDurationInstance],
    transition_global_id: u32,
) -> Option<f32> {
    transition_durations
        .iter()
        .find(|duration| duration.transition_global_id == transition_global_id)
        .map(StateMachineTransitionDurationInstance::value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransitionAllowance {
    No,
    WaitingForExit,
    Yes,
}
