use super::*;

#[derive(Debug, Clone)]
pub(crate) struct StateMachineLayerInstance {
    /// C++ keys trigger consumption by `StateMachineLayerInstance*`. A fresh
    /// monotonic token gives every Rust layer occurrence the same identity
    /// boundary, including cloned state machines that share a retained VM.
    view_model_trigger_layer_id: u64,
    any_state: Option<RuntimeStateInstance>,
    current_state: Option<RuntimeStateInstance>,
    state_from: Option<RuntimeStateInstance>,
    transition_duration_seconds: f32,
    transition_mix: f32,
    transition_mix_from: f32,
    hold_animation_from: bool,
    hold_animation: Option<(RuntimeLinearAnimationHandle, f32)>,
    active_transition: Option<RuntimeStateTransitionHandle>,
    transition_completed: bool,
    transition_animation_reset: Option<AnimationReset>,
    waiting_for_exit: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineLayerAdvance {
    pub(crate) changed_state: bool,
    pub(crate) keep_going: bool,
}

impl StateMachineLayerInstance {
    pub(crate) fn new(
        layer: &RuntimeStateMachineLayer,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
    ) -> Self {
        let mut any_state = layer.any_state_index.and_then(|state_index| {
            RuntimeStateInstance::make(layer, state_index, artboard, inputs, bindable_numbers)
        });
        let mut current_state = layer.entry_state_index.and_then(|state_index| {
            RuntimeStateInstance::make(layer, state_index, artboard, inputs, bindable_numbers)
        });
        if let Some(any_state) = any_state.as_mut() {
            any_state.prepare_key_frame_data_binds(key_frame_data_bind_graphs);
        }
        if let Some(current_state) = current_state.as_mut() {
            current_state.prepare_key_frame_data_binds(key_frame_data_bind_graphs);
        }
        Self {
            view_model_trigger_layer_id: next_view_model_trigger_layer_id(),
            any_state,
            current_state,
            state_from: None,
            transition_duration_seconds: 0.0,
            transition_mix: 1.0,
            transition_mix_from: 1.0,
            hold_animation_from: false,
            hold_animation: None,
            active_transition: None,
            transition_completed: false,
            transition_animation_reset: None,
            waiting_for_exit: false,
        }
    }

    pub(crate) fn refresh_view_model_trigger_layer_id(&mut self) {
        self.view_model_trigger_layer_id = next_view_model_trigger_layer_id();
    }

    #[cfg(test)]
    pub(crate) fn view_model_trigger_layer_id(&self) -> u64 {
        self.view_model_trigger_layer_id
    }

    pub(crate) fn has_current_animation(&self) -> bool {
        self.current_state
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
            .is_some()
    }

    pub(crate) fn current_animation(&self) -> Option<&LinearAnimationInstance> {
        self.current_state
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
    }

    pub(crate) fn perform_initial_entry_actions(
        &self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<(), ScriptError> {
        let Some(state) = self
            .current_state
            .as_ref()
            .and_then(|state| state.state(layer))
        else {
            return Ok(());
        };
        state.perform_fire_actions(
            StateMachineFireOccurrence::AtStart,
            executor,
            &mut *targets.reported_events,
        );
        state.perform_listener_actions(
            StateMachineFireOccurrence::AtStart,
            artboard,
            targets.reborrow(),
            executor,
        )?;
        Ok(())
    }

    fn current_state_index(&self) -> Option<usize> {
        self.current_state
            .as_ref()
            .map(RuntimeStateInstance::state_index)
    }

    pub(crate) fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
        elapsed_seconds: f32,
        data_context_present: bool,
        layer_index: usize,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<StateMachineLayerAdvance, ScriptError> {
        let key_frame_data_bind_keep_going =
            self.advance_key_frame_data_binds(key_frame_data_bind_graphs, elapsed_seconds);
        self.advance_current_animation(
            artboard,
            layer,
            elapsed_seconds,
            targets.inputs,
            targets.bindable_numbers,
            targets.reported_events,
        );
        let input_changed = self.update_transition_mix(
            artboard,
            layer,
            elapsed_seconds,
            targets.reborrow(),
            executor,
        )?;
        self.advance_transition_source_animation(
            artboard,
            layer,
            elapsed_seconds,
            targets.inputs,
            targets.bindable_numbers,
            targets.reported_events,
        );
        self.apply_animations(artboard, layer, key_frame_data_bind_graphs);

        let mut changed_state = false;
        // Pinned C++ tests the limit after each successful update. Its loop
        // therefore applies updates numbered 0 through 100, then returns
        // `false` immediately on the 101st success without clearing spilled
        // time (`state_machine_instance.cpp:227-259`).
        for iteration in 0..=100 {
            if !self.update_state(
                artboard,
                layer,
                key_frame_data_bind_graphs,
                data_context_present,
                layer_index,
                targets.reborrow(),
                executor,
            )? {
                break;
            }
            changed_state = true;
            self.apply_animations(artboard, layer, key_frame_data_bind_graphs);
            if iteration == 100 {
                return Ok(StateMachineLayerAdvance {
                    changed_state: true,
                    keep_going: false,
                });
            }
        }
        if let Some(current_state) = self.current_state.as_mut() {
            current_state.clear_spilled_time();
        }

        Ok(StateMachineLayerAdvance {
            changed_state,
            keep_going: changed_state
                || input_changed
                || key_frame_data_bind_keep_going
                || self.is_transitioning()
                || self.waiting_for_exit
                || self
                    .current_state
                    .as_ref()
                    .is_some_and(RuntimeStateInstance::keep_going),
        })
    }

    pub(crate) fn reset_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<(), ScriptError> {
        // C++ removes occurrence-owned keyframe binds before deleting source
        // and current. Rust keyframe binds are owned by the animation
        // occurrence itself, so dropping these two Options performs the same
        // teardown and makes the C++ alias guards unrepresentable.
        self.state_from = None;
        self.current_state = None;

        let Some(entry_state_index) = layer.entry_state_index else {
            return Ok(());
        };
        self.current_state = RuntimeStateInstance::make(
            layer,
            entry_state_index,
            artboard,
            targets.inputs,
            targets.bindable_numbers,
        );
        if let Some(current_state) = self.current_state.as_mut() {
            current_state.prepare_key_frame_data_binds(key_frame_data_bind_graphs);
        }
        let Some(entry_state) = self
            .current_state
            .as_ref()
            .and_then(|state| state.state(layer))
        else {
            return Ok(());
        };
        entry_state.perform_fire_actions(
            StateMachineFireOccurrence::AtStart,
            executor,
            &mut *targets.reported_events,
        );
        entry_state.perform_listener_actions(
            StateMachineFireOccurrence::AtStart,
            artboard,
            targets.reborrow(),
            executor,
        )?;
        Ok(())
    }

    pub(crate) fn update_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
        data_context_present: bool,
        layer_index: usize,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<bool, ScriptError> {
        self.refresh_resolved_transition_duration(artboard, layer, targets.transition_durations);
        if self.is_transitioning()
            && !self
                .active_transition
                .and_then(|handle| handle.resolve(layer))
                .is_some_and(RuntimeStateTransition::enable_early_exit)
        {
            return Ok(false);
        }
        self.waiting_for_exit = false;
        if self.try_change_state(
            artboard,
            layer,
            key_frame_data_bind_graphs,
            self.any_state
                .as_ref()
                .map(RuntimeStateInstance::state_index),
            data_context_present,
            layer_index,
            targets.reborrow(),
            executor,
        )? {
            return Ok(true);
        }
        self.try_change_state(
            artboard,
            layer,
            key_frame_data_bind_graphs,
            self.current_state
                .as_ref()
                .map(RuntimeStateInstance::state_index),
            data_context_present,
            layer_index,
            targets,
            executor,
        )
    }

    fn try_change_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
        state_index: Option<usize>,
        data_context_present: bool,
        layer_index: usize,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<bool, ScriptError> {
        let Some(state_index) = state_index else {
            return Ok(false);
        };
        let Some(state) = layer.states.get(state_index) else {
            return Ok(false);
        };

        if state.uses_random_transition_selection() {
            let random_transition = {
                let context = targets.evaluation_context(
                    data_context_present,
                    layer_index,
                    self.view_model_trigger_layer_id,
                );
                self.find_random_transition(
                    &context,
                    artboard,
                    state,
                    state_index,
                    targets.inputs,
                    executor,
                )
            };
            let Some((transition_index, state_to_index)) = random_transition else {
                return Ok(false);
            };
            let transition = &state.transitions[transition_index];
            transition.use_inputs(
                targets.inputs,
                executor,
                layer_index,
                self.view_model_trigger_layer_id,
            );
            self.change_state(
                artboard,
                layer,
                key_frame_data_bind_graphs,
                RuntimeStateTransitionHandle::new(state_index, transition_index),
                transition,
                state_to_index,
                targets,
                executor,
            )?;
            return Ok(true);
        }

        for (transition_index, transition) in state.transitions.iter().enumerate() {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self
                .current_state
                .as_ref()
                .is_some_and(|state| state.is_same_definition(state_to_index))
            {
                continue;
            }
            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index() == Some(state_index),
            );
            let allowance = {
                let context = targets.evaluation_context(
                    data_context_present,
                    layer_index,
                    self.view_model_trigger_layer_id,
                );
                if transition.direct_input_conditions_only {
                    transition.allow_direct_inputs(&context, targets.inputs, animation_from)
                } else {
                    transition.allow(&context, artboard, targets.inputs, executor, animation_from)
                }
            };
            match allowance {
                TransitionAllowance::No => continue,
                TransitionAllowance::WaitingForExit => {
                    self.waiting_for_exit = true;
                    continue;
                }
                TransitionAllowance::Yes => {
                    self.waiting_for_exit = false;
                }
            }
            transition.use_inputs(
                targets.inputs,
                executor,
                layer_index,
                self.view_model_trigger_layer_id,
            );
            self.change_state(
                artboard,
                layer,
                key_frame_data_bind_graphs,
                RuntimeStateTransitionHandle::new(state_index, transition_index),
                transition,
                state_to_index,
                targets,
                executor,
            )?;
            return Ok(true);
        }
        Ok(false)
    }

    fn find_random_transition(
        &mut self,
        context: &TransitionEvaluationContext<'_>,
        artboard: &ArtboardInstance,
        state: &RuntimeLayerState,
        state_index: usize,
        inputs: &[StateMachineInputInstance],
        executor: &dyn RuntimeScheduledListenerActionExecutor,
    ) -> Option<(usize, usize)> {
        let mut weighted_candidates = Vec::new();
        let mut total_weight = 0_u64;
        let mut waiting_for_exit = false;

        for (transition_index, transition) in state.transitions.iter().enumerate() {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self
                .current_state
                .as_ref()
                .is_some_and(|state| state.is_same_definition(state_to_index))
            {
                continue;
            }

            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index() == Some(state_index),
            );
            let allowance = if transition.direct_input_conditions_only {
                transition.allow_direct_inputs(context, inputs, animation_from)
            } else {
                transition.allow(context, artboard, inputs, executor, animation_from)
            };
            match allowance {
                TransitionAllowance::No => {}
                TransitionAllowance::WaitingForExit => {
                    waiting_for_exit = true;
                }
                TransitionAllowance::Yes => {
                    total_weight = total_weight.saturating_add(transition.random_weight);
                    weighted_candidates.push((
                        transition_index,
                        state_to_index,
                        transition.random_weight,
                    ));
                }
            }
        }

        if total_weight == 0 {
            self.waiting_for_exit = waiting_for_exit;
            return None;
        }

        let random_weight = Self::random_transition_value() * total_weight as f64;
        let mut current_weight = 0.0_f64;
        for (transition_index, state_to_index, transition_weight) in weighted_candidates {
            current_weight += transition_weight as f64;
            if current_weight > random_weight {
                self.waiting_for_exit = false;
                return Some((transition_index, state_to_index));
            }
        }

        self.waiting_for_exit = waiting_for_exit;
        None
    }

    fn random_transition_value() -> f64 {
        0.0
    }

    fn current_transition_animation<'a>(
        &'a self,
        artboard: &'a ArtboardInstance,
        transition: &RuntimeStateTransition,
        is_current_state: bool,
    ) -> Option<RuntimeTransitionAnimationRef<'a>> {
        if !is_current_state {
            return None;
        }

        let animation_instance = self
            .current_state
            .as_ref()?
            .transition_animation(transition.exit_blend_animation_index)?;
        let animation = artboard.linear_animation_instance_definition(animation_instance)?;
        Some(RuntimeTransitionAnimationRef {
            instance: animation_instance,
            animation,
        })
    }

    fn change_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
        transition_handle: RuntimeStateTransitionHandle,
        transition: &RuntimeStateTransition,
        state_to_index: usize,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<(), ScriptError> {
        // Pinned C++ clears the prior transition's reset before it runs the
        // outgoing state's end actions and constructs the replacement
        // occurrence (`state_machine_instance.cpp:528-540`).
        self.transition_animation_reset = None;
        let previous_state = self.current_state.take();
        let previous_state_index = previous_state
            .as_ref()
            .map(RuntimeStateInstance::state_index);
        let previous_spilled_time = previous_state
            .as_ref()
            .map(RuntimeStateInstance::spilled_time)
            .unwrap_or(0.0);
        let previous_mix = self.transition_mix;
        if let Some(previous_state) =
            previous_state_index.and_then(|state_index| layer.states.get(state_index))
        {
            previous_state.perform_fire_actions(
                StateMachineFireOccurrence::AtEnd,
                executor,
                &mut *targets.reported_events,
            );
            previous_state.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                artboard,
                targets.reborrow(),
                executor,
            )?;
        }

        self.current_state = RuntimeStateInstance::make(
            layer,
            state_to_index,
            artboard,
            targets.inputs,
            targets.bindable_numbers,
        );
        if let Some(current_state) = self.current_state.as_mut() {
            current_state.prepare_key_frame_data_binds(key_frame_data_bind_graphs);
        }
        if let Some(current_state) = layer.states.get(state_to_index) {
            current_state.perform_fire_actions(
                StateMachineFireOccurrence::AtStart,
                executor,
                &mut *targets.reported_events,
            );
            current_state.perform_listener_actions(
                StateMachineFireOccurrence::AtStart,
                artboard,
                targets.reborrow(),
                executor,
            )?;
        }

        self.active_transition = Some(transition_handle);
        let previous_runtime_animation = previous_state
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
            .and_then(|animation_instance| {
                artboard.linear_animation_instance_definition(animation_instance)
            });
        let duration_override =
            transition_duration_value(targets.transition_durations, transition.global_id);
        let transition_duration_seconds =
            transition.transition_duration_seconds(previous_runtime_animation, duration_override);
        transition.perform_fire_actions(
            StateMachineFireOccurrence::AtStart,
            executor,
            &mut *targets.reported_events,
        );
        transition.perform_listener_actions(
            StateMachineFireOccurrence::AtStart,
            artboard,
            targets.reborrow(),
            executor,
        )?;

        self.transition_completed = transition_duration_seconds == 0.0;
        if transition_duration_seconds == 0.0 {
            transition.perform_fire_actions(
                StateMachineFireOccurrence::AtEnd,
                executor,
                &mut *targets.reported_events,
            );
            transition.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                artboard,
                targets.reborrow(),
                executor,
            )?;
        }

        let mut reset_animation_indices = Vec::new();
        if let Some(animation) = previous_state
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
        {
            reset_animation_indices.push(animation.animation_index());
        }
        if let Some(animation) = self
            .current_state
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
        {
            reset_animation_indices.push(animation.animation_index());
        }
        // A transition interruption releases the older transition-source
        // occurrence before the outgoing current occurrence becomes the new
        // source (`state_machine_instance.cpp:573-580`).
        self.state_from = None;
        self.state_from = previous_state;
        if previous_state_index.is_some() {
            self.transition_duration_seconds = transition_duration_seconds;
            self.transition_animation_reset = (!self.transition_completed).then(|| {
                AnimationResetFactory::from_animation_indices(
                    artboard,
                    &reset_animation_indices,
                    false,
                )
            });

            if transition.has_exit_time()
                && let Some(animation_instance) = self
                    .state_from
                    .as_mut()
                    .and_then(RuntimeStateInstance::plain_animation_mut)
                && let Some(animation) =
                    artboard.linear_animation_instance_definition(animation_instance)
            {
                if transition.pause_on_exit() {
                    animation_instance.set_time(
                        animation,
                        transition.exit_time_seconds(Some(animation), true),
                    );
                }
                self.hold_animation = Some((
                    animation_instance.animation_handle(),
                    animation_instance.time(),
                ));
            }
            self.transition_mix_from = previous_mix;
            // C++ only updates this hold flag when the previous mix was
            // nonzero. Preserve the old value at zero rather than replacing
            // that branch with a simpler assignment.
            if previous_mix != 0.0 {
                self.hold_animation_from = transition.pause_on_exit();
            }
            if previous_spilled_time != 0.0
                && let Some(current_state) = self.current_state.as_mut()
            {
                current_state.advance(
                    layer,
                    artboard,
                    targets.inputs,
                    targets.bindable_numbers,
                    previous_spilled_time,
                    targets.reported_events,
                );
            }
            self.transition_mix = 0.0;
            self.update_transition_mix(artboard, layer, 0.0, targets.reborrow(), executor)?;
        } else {
            self.clear_transition_source();
        }
        Ok(())
    }

    fn clear_transition_source(&mut self) {
        self.state_from = None;
        self.transition_duration_seconds = 0.0;
        self.transition_mix = 1.0;
        self.transition_mix_from = 1.0;
        self.hold_animation_from = false;
        self.hold_animation = None;
        self.active_transition = None;
        self.transition_completed = false;
        self.transition_animation_reset = None;
    }

    fn is_transitioning(&self) -> bool {
        self.has_transition_source()
            && self.transition_duration_seconds != 0.0
            && self.transition_mix < 1.0
    }

    fn has_transition_source(&self) -> bool {
        self.state_from.is_some()
    }

    fn update_transition_mix(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    ) -> Result<bool, ScriptError> {
        self.refresh_resolved_transition_duration(artboard, layer, targets.transition_durations);
        if !self.has_transition_source() || self.transition_duration_seconds == 0.0 {
            self.transition_mix = 1.0;
            return Ok(false);
        }
        self.transition_mix = (self.transition_mix
            + elapsed_seconds / self.transition_duration_seconds)
            .clamp(0.0, 1.0);
        if self.transition_mix == 1.0 && !self.transition_completed {
            self.transition_completed = true;
            self.transition_animation_reset = None;
            let Some(transition) = self
                .active_transition
                .and_then(|handle| handle.resolve(layer))
            else {
                return Ok(false);
            };
            transition.perform_fire_actions(
                StateMachineFireOccurrence::AtEnd,
                executor,
                &mut *targets.reported_events,
            );
            transition.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                artboard,
                targets.reborrow(),
                executor,
            )
        } else {
            Ok(false)
        }
    }

    fn refresh_resolved_transition_duration(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        transition_durations: &[StateMachineTransitionDurationInstance],
    ) {
        let Some(transition) = self
            .active_transition
            .and_then(|handle| handle.resolve(layer))
        else {
            return;
        };
        let animation = self
            .state_from
            .as_ref()
            .and_then(RuntimeStateInstance::plain_animation)
            .and_then(|instance| artboard.linear_animation_instance_definition(instance));
        let duration_override =
            transition_duration_value(transition_durations, transition.global_id);
        self.transition_duration_seconds =
            transition.transition_duration_seconds(animation, duration_override);
    }

    fn advance_current_animation(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.current_state.as_mut().is_some_and(|state| {
            state.advance(
                layer,
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            )
        })
    }

    fn advance_transition_source_animation(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if !self.is_transitioning() {
            return false;
        }
        if self.hold_animation_from {
            return false;
        }
        self.state_from.as_mut().is_some_and(|state| {
            state.advance(
                layer,
                artboard,
                inputs,
                bindable_numbers,
                elapsed_seconds,
                reported_events,
            )
        })
    }

    fn apply_animations(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        key_frame_data_bind_graphs: &[Option<crate::RuntimeDataBindGraph>],
    ) -> bool {
        self.prepare_key_frame_data_binds(key_frame_data_bind_graphs);
        let mut changed = self
            .transition_animation_reset
            .as_ref()
            .is_some_and(|reset| reset.apply(artboard));
        if let Some((animation_handle, hold_time)) = self.hold_animation.take() {
            changed |= artboard.apply_linear_animation(
                animation_handle.index(),
                hold_time,
                self.transition_mix_from,
            );
        }
        let interpolator = self
            .active_transition
            .and_then(|handle| handle.resolve(layer))
            .and_then(|transition| transition.interpolator);
        if self.state_from.is_some() && self.transition_mix < 1.0 {
            let mix_from = interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix_from))
                .unwrap_or(self.transition_mix_from);
            if let Some(state_from) = self.state_from.as_ref() {
                changed |= state_from.apply(artboard, mix_from);
            }
        }
        let mix = interpolator
            .map(|interpolator| interpolator.transform(self.transition_mix))
            .unwrap_or(self.transition_mix);
        if let Some(current_state) = self.current_state.as_ref() {
            changed |= current_state.apply(artboard, mix);
        }
        changed
    }

    fn prepare_key_frame_data_binds(&mut self, graphs: &[Option<crate::RuntimeDataBindGraph>]) {
        if graphs.is_empty() {
            return;
        }
        if let Some(current_state) = self.current_state.as_mut() {
            current_state.prepare_key_frame_data_binds(graphs);
        }
        if let Some(state_from) = self.state_from.as_mut() {
            state_from.prepare_key_frame_data_binds(graphs);
        }
    }

    fn advance_key_frame_data_binds(
        &mut self,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
        elapsed_seconds: f32,
    ) -> bool {
        if graphs.is_empty() {
            return false;
        }
        let mut keep_going = false;
        Self::for_each_animation_instance_mut(self, |instance| {
            let prototype = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref);
            keep_going |= instance.advance_key_frame_data_binds(prototype, elapsed_seconds);
        });
        keep_going
    }

    fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        if let Some(current_state) = self.current_state.as_mut() {
            current_state.for_each_animation_instance_mut(&mut callback);
        }
        if let Some(state_from) = self.state_from.as_mut() {
            state_from.for_each_animation_instance_mut(&mut callback);
        }
    }
}
