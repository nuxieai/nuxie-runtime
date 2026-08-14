use nuxie_binary::RuntimeFile;

#[cfg(feature = "tools")]
use super::RuntimeNestedRemapAnimationReport;
use super::{
    ArtboardInstance, RuntimeNestedAnimationInstance, RuntimeNestedArtboardInstance,
    RuntimeScriptAdvanceMode,
};
use crate::RuntimeOwnedViewModelInstance;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::scripting::ScriptError;
use crate::state_machine::{
    RuntimeNestedStateMachineReport, StateMachineInstance, StateMachineReportedEvent,
};

impl ArtboardInstance {
    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_nested_artboards_collect_events(elapsed_seconds, None)
    }

    /// Report imported nested-state-machine occurrence ownership and empty
    /// forwarding behavior for pinned-C++ differentials.
    #[doc(hidden)]
    pub fn runtime_nested_state_machine_reports(&self) -> Vec<RuntimeNestedStateMachineReport> {
        self.nested_artboards
            .values()
            .flat_map(|nested| {
                nested
                    .animations
                    .iter()
                    .filter_map(|animation| match animation {
                        RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                            Some(occurrence.empty_contract_report(&nested.child))
                        }
                        RuntimeNestedAnimationInstance::Simple { .. }
                        | RuntimeNestedAnimationInstance::Remap { .. } => None,
                    })
            })
            .collect()
    }

    /// Report mounted remap occurrence time for pinned-C++ differentials.
    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn runtime_nested_remap_animation_reports(&self) -> Vec<RuntimeNestedRemapAnimationReport> {
        self.nested_artboards
            .iter()
            .flat_map(|(host_local_id, nested)| {
                nested
                    .animations
                    .iter()
                    .filter_map(|animation| match animation {
                        RuntimeNestedAnimationInstance::Remap {
                            local_id,
                            animation,
                            ..
                        } => Some(RuntimeNestedRemapAnimationReport {
                            host_local_id: *host_local_id,
                            local_id: *local_id,
                            animation_time: animation.time(),
                        }),
                        RuntimeNestedAnimationInstance::Simple { .. }
                        | RuntimeNestedAnimationInstance::StateMachine(_) => None,
                    })
            })
            .collect()
    }

    pub fn try_visit_nested_artboard_instances_mut<E>(
        &mut self,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        self.try_visit_nested_artboard_instances_mut_at_depth(1, visitor)
    }

    fn try_visit_nested_artboard_instances_mut_at_depth<E>(
        &mut self,
        depth: usize,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        for nested in self.nested_artboards.values_mut() {
            visitor(depth, nested.child.graph_global_id, nested.child.as_mut())?;
            nested
                .child
                .try_visit_nested_artboard_instances_mut_at_depth(depth + 1, visitor)?;
        }
        Ok(())
    }

    pub fn bind_nested_artboard_owned_context_for_graph(
        &mut self,
        file: &RuntimeFile,
        graph_global_id: u32,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let mut changed = false;
        for nested in self.nested_artboards.values_mut() {
            if nested.child.graph_global_id == graph_global_id {
                let context_chain: [&[usize]; 1] = [&[]];
                changed |=
                    nested.bind_owned_view_model_animation_contexts(file, context, &context_chain);
                changed |= nested
                    .child
                    .bind_owned_view_model_artboard_context(file, context);
            } else {
                changed |= nested.child.bind_nested_artboard_owned_context_for_graph(
                    file,
                    graph_global_id,
                    context,
                );
            }
        }
        changed
    }

    pub fn set_nested_script_owned_context_for_graph(
        &mut self,
        graph_global_id: u32,
        context: RuntimeOwnedViewModelInstance,
    ) {
        self.nested_script_owned_contexts
            .insert(graph_global_id, context);
    }

    pub fn rebind_nested_script_owned_contexts(&mut self, file: &RuntimeFile) -> bool {
        let contexts = self
            .nested_script_owned_contexts
            .iter()
            .map(|(graph_global_id, context)| (*graph_global_id, context.clone()))
            .collect::<Vec<_>>();
        let mut changed = false;
        for (graph_global_id, context) in contexts {
            changed |=
                self.bind_nested_artboard_owned_context_for_graph(file, graph_global_id, &context);
        }
        changed
    }

    pub fn advance_nested_artboards_with_state_machine(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> bool {
        StateMachineInstance::dispatch_nested_event_sources_with(
            self,
            state_machine,
            |artboard, nested_event_dispatch| {
                let _ = artboard
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        true,
                        &mut RuntimeScriptAdvanceMode::Disabled,
                        None,
                        Some(nested_event_dispatch),
                    )
                    .expect("disabled script dispatch cannot fail");
                Ok(())
            },
        )
        .expect("nested-artboard collection cannot dispatch scripts")
    }
}

impl RuntimeNestedArtboardInstance {
    /// A same-layer authored VMI write can be consumed by the mounted child
    /// without publishing a second layout assignment back into its host.
    /// Advance the transfer fence to that consumed child generation so the
    /// parent-owned Yoga snapshot remains authoritative.
    pub(crate) fn acknowledge_consumed_child_layout_revision(&mut self) {
        if let Some(key) = self.layout_data_transfer_key.as_mut() {
            key.child_layout_revision = self.child.layout_revision();
        }
    }

    pub(in crate::artboard) fn install_external_focus_domain(
        &mut self,
        parent_focus: &crate::focus::RuntimeFocusTree,
    ) {
        let child_identity = self.child.instance_identity();
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            occurrence.install_external_focus(parent_focus, child_identity);
        }
        self.child.install_external_focus_domain(parent_focus);
    }

    pub(in crate::artboard) fn reuse_owned_stateful_view_model_context(
        &mut self,
        existing: &Self,
    ) -> bool {
        if self.stateful_view_model_instance_local.is_some()
            || existing.stateful_view_model_instance_local.is_some()
        {
            return false;
        }
        let Some(replacement_context) = self.stateful_view_model_context.as_ref() else {
            return false;
        };
        let Some(existing_context) = existing.stateful_view_model_context.as_ref() else {
            return false;
        };
        if replacement_context.borrow().view_model_index()
            != existing_context.borrow().view_model_index()
        {
            return false;
        }
        self.stateful_view_model_context = Some(existing_context.clone());
        true
    }

    pub(in crate::artboard) fn has_ongoing_work(&self) -> bool {
        if self.is_paused {
            return false;
        }
        self.animations
            .iter()
            .any(|animation| animation.has_ongoing_work(&self.child))
            || self.child.has_ongoing_nested_work()
    }

    pub(crate) fn bind_owned_view_model_animation_contexts(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            changed |= occurrence.bind_owned_view_model_context_chain(file, context, context_chain);
        }
        changed
    }

    pub(crate) fn bind_owned_view_model_animation_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            changed |= occurrence.bind_owned_data_context(data_context);
        }
        changed
    }

    /// Bind one mounted occurrence in the same order as C++
    /// `NestedArtboard::bindStateful`: first install the context on the child
    /// Artboard, then forward the child's context to every nested state
    /// machine (`src/nested_artboard.cpp:156-185`).
    pub(crate) fn bind_owned_view_model_occurrence_data_context(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        allow_full_context_bindings: bool,
    ) -> bool {
        let mut changed = self.child.bind_owned_view_model_artboard_data_context(
            file,
            data_context,
            true,
            allow_full_context_bindings,
        );
        changed |= self.bind_owned_view_model_animation_data_context(data_context);
        changed
    }

    pub(in crate::artboard) fn begin_advance(&mut self, elapsed_seconds: f32) -> Result<f32, bool> {
        if self.is_paused {
            return Err(false);
        }

        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(elapsed_seconds);
        if local_elapsed_seconds == 0.0 && self.quantize >= 0.0 {
            // C++ returns before advancing nested animations on a quantized
            // NewFrame skip, then unconditionally probes nested state machines
            // during the following non-NewFrame outer pass.
            return Err(true);
        }
        Ok(local_elapsed_seconds)
    }

    pub(in crate::artboard) fn advance_after_animation_owners(
        &mut self,
        parent_artboard: &mut ArtboardInstance,
        parent_host_local: usize,
        local_elapsed_seconds: f32,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        mut ancestor_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
    ) -> Result<bool, ScriptError> {
        // C++ advances the ENTIRE nested subtree before any data-bind pass
        // reaches it: `NestedArtboard::advanceComponent` only advances
        // animations and `advanceInternal` (src/nested_artboard.cpp:965-1008),
        // while the data binds — including the owned-path target-to-source
        // pulls — run later through `Artboard::updateDataBinds` recursing
        // artboard hosts first (src/artboard.cpp:1195-1201, called from
        // `updatePass` at src/artboard.cpp:1420). Advancing this child's
        // binds before its own nested artboards let a grandchild state
        // machine observe a reverse write one pass earlier than C++ (the
        // db_health_tracker blend consumed the pulled value on the first
        // frame where C++ still blends the pre-pull value).
        let animations = &mut self.animations;
        let mut dispatch_nested_source =
            |child: &mut ArtboardInstance,
             host_local: usize,
             events: &[StateMachineReportedEvent]| {
                match ancestor_dispatch.as_deref_mut() {
                    Some(dispatch) => {
                        StateMachineInstance::dispatch_nested_events_to_animation_owners(
                            parent_artboard,
                            parent_host_local,
                            animations,
                            child,
                            host_local,
                            events,
                            nested_events.as_deref_mut(),
                            Some(dispatch),
                        )
                    }
                    None => StateMachineInstance::dispatch_nested_events_to_animation_owners(
                        parent_artboard,
                        parent_host_local,
                        animations,
                        child,
                        host_local,
                        events,
                        nested_events.as_deref_mut(),
                        None,
                    ),
                }
            };
        let child_result = self
            .child
            .advance_retained_components_collect_events_with_scripts(
                local_elapsed_seconds,
                true,
                script_mode,
                None,
                Some(&mut dispatch_nested_source),
            );
        let mut changed = child_result.as_ref().copied().unwrap_or(false);
        drop(dispatch_nested_source);
        // Mirrors C++ src/nested_artboard.cpp NestedArtboard::updateDataBinds.
        changed |= self
            .child
            .advance_artboard_data_binds_with_elapsed(local_elapsed_seconds);
        if let Err(error) = child_result {
            return Err(error);
        }
        Ok(changed)
    }

    pub(in crate::artboard) fn reset_outer_state_machine_changed_state_counts(&mut self) {
        for animation in &mut self.animations {
            if let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation {
                if let Some(state_machine) = occurrence.state_machine_mut() {
                    state_machine.reset_changed_state_count_for_outer_settlement();
                }
            }
        }
        self.child
            .reset_outer_state_machine_changed_state_counts(&mut []);
    }

    /// Advance the non-`NewFrame` portion of C++
    /// `NestedArtboard::advanceComponent`: only state machines whose probe
    /// changes state are applied, followed by the child artboard's advancing
    /// components and data binds.
    pub(in crate::artboard) fn advance_outer_update(&mut self) -> bool {
        if self.is_paused {
            return false;
        }

        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(0.0);
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            let Some(state_machine) = occurrence.state_machine_mut() else {
                continue;
            };
            if self.child.try_change_state_machine_instance(state_machine) {
                changed = true;
                changed |= self.child.advance_state_machine_instance_after_state_probe(
                    state_machine,
                    local_elapsed_seconds,
                );
            }
        }
        changed |= self
            .child
            .advance_outer_update_components_for_state_machine_settlement();
        changed
    }

    // Mirrors src/nested_artboard.cpp NestedArtboard::calculateLocalElapsedSeconds.
    pub(in crate::artboard) fn calculate_local_elapsed_seconds(
        &mut self,
        elapsed_seconds: f32,
    ) -> f32 {
        let mut local_elapsed_seconds =
            elapsed_seconds * if self.speed >= 0.0 { self.speed } else { 1.0 };
        if self.quantize >= 0.0 {
            self.cumulated_seconds += local_elapsed_seconds;
            let quantized_seconds = 1.0 / self.quantize;
            if self.cumulated_seconds > quantized_seconds {
                local_elapsed_seconds =
                    (self.cumulated_seconds / quantized_seconds).floor() * quantized_seconds;
                self.cumulated_seconds -= local_elapsed_seconds;
            } else {
                local_elapsed_seconds = 0.0;
            }
        }
        local_elapsed_seconds
    }

    pub(in crate::artboard) fn set_root_opacity(&mut self, opacity: f32) -> bool {
        self.child.set_host_opacity(opacity)
    }

    pub(in crate::artboard) fn set_remap_time(&mut self, remap_local_id: usize, time: f32) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                ..
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id {
                continue;
            }
            let Some(linear_animation) = self.child.linear_animation(animation.animation_index())
            else {
                return false;
            };
            let seconds = linear_animation
                .global_to_local_seconds(linear_animation.duration_seconds() * time);
            animation.set_time(linear_animation, seconds);
            return true;
        }
        false
    }

    pub(in crate::artboard) fn set_animation_mix(&mut self, local_id: usize, value: f32) -> bool {
        for animation in &mut self.animations {
            let (animation_local_id, mix) = match animation {
                RuntimeNestedAnimationInstance::Simple { local_id, mix, .. }
                | RuntimeNestedAnimationInstance::Remap { local_id, mix, .. } => (local_id, mix),
                RuntimeNestedAnimationInstance::StateMachine(_) => continue,
            };
            if *animation_local_id != local_id || *mix == value {
                continue;
            }
            *mix = value;
            return true;
        }
        false
    }

    pub(in crate::artboard) fn set_simple_animation_speed(
        &mut self,
        local_id: usize,
        value: f32,
    ) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Simple {
                local_id: animation_local_id,
                speed,
                ..
            } = animation
            else {
                continue;
            };
            if *animation_local_id != local_id || *speed == value {
                continue;
            }
            *speed = value;
            return true;
        }
        false
    }

    pub(in crate::artboard) fn set_simple_animation_is_playing(
        &mut self,
        local_id: usize,
        value: bool,
    ) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Simple {
                local_id: animation_local_id,
                is_playing,
                ..
            } = animation
            else {
                continue;
            };
            if *animation_local_id != local_id || *is_playing == value {
                continue;
            }
            *is_playing = value;
            return true;
        }
        false
    }

    pub(in crate::artboard) fn advance_remap(&mut self, remap_local_id: usize) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                mix,
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id || *mix == 0.0 {
                continue;
            }
            return self.child.apply_linear_animation_instance(animation, *mix);
        }
        false
    }

    pub(in crate::artboard) fn set_is_paused(&mut self, value: bool) -> bool {
        if self.is_paused == value {
            return false;
        }
        self.is_paused = value;
        true
    }

    pub(in crate::artboard) fn set_speed(&mut self, value: f32) -> bool {
        if self.speed == value {
            return false;
        }
        self.speed = value;
        true
    }

    pub(in crate::artboard) fn set_quantize(&mut self, value: f32) -> bool {
        if self.quantize == value {
            return false;
        }
        self.quantize = value;
        true
    }
}
