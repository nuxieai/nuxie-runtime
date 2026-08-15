// State-machine instance integration for the C++ `data_bind_container.cpp` source.
use super::state_machine_instance::apply_scripted_input_update;
use super::*;
impl StateMachineInstance {
    pub(super) fn teardown_bind_occurrences(&mut self) {
        self.record_drop_phase("binds");
        for layer in &mut self.layers {
            layer.remove_key_frame_data_binds();
        }
        self.owned_data_context = None;
        self.data_bind_occurrences.clear();
        self.data_bind_container = RuntimeDataBindContainerQueue::default();
        self.key_frame_data_bind_graphs.clear();
        self.data_bind_graph.sources.clear();
        self.data_bind_graph.targets.clear();
        self.data_bind_graph.default_view_model_bindings.clear();
        self.data_bind_graph.imported_view_model_overrides.clear();
        self.scripted_object_bindings.clear();
    }
    fn key_frame_data_bind_occurrence_ids(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
    ) -> Vec<crate::animation::RuntimeKeyFrameDataBindOccurrenceId> {
        let (layers, graphs, next_id) = (
            &mut self.layers,
            &self.key_frame_data_bind_graphs,
            &mut self.next_key_frame_data_bind_occurrence_id,
        );
        for layer in &mut *layers {
            // Snapshot Clone deliberately drops mutable graph occurrences.
            // Rebuild them from the immutable prototype before collecting
            // typed owner-local enrollment identities.
            layer.ensure_key_frame_data_binds(graphs);
            layer.enroll_unassigned_key_frame_data_binds(next_id);
        }
        let mut ids = Vec::new();
        for layer in layers {
            layer.collect_key_frame_data_bind_occurrence_ids(enrollment, &mut ids);
        }
        ids.sort_unstable();
        ids
    }
    pub(super) fn prepare_key_frame_data_bind_enrollment(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
    ) -> bool {
        let ids = self.key_frame_data_bind_occurrence_ids(enrollment);
        let (layers, graphs) = (&mut self.layers, &self.key_frame_data_bind_graphs);
        let mut changed = false;
        for id in ids {
            for layer in &mut *layers {
                if let Some(result) = layer.prepare_key_frame_data_bind_occurrence(id, graphs) {
                    changed |= result;
                    break;
                }
            }
        }
        changed
    }
    pub(super) fn advance_key_frame_data_bind_enrollment(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
        elapsed_seconds: f32,
    ) -> bool {
        let ids = self.key_frame_data_bind_occurrence_ids(enrollment);
        let (layers, graphs) = (&mut self.layers, &self.key_frame_data_bind_graphs);
        let mut keep_going = false;
        for id in ids {
            for layer in &mut *layers {
                if let Some(result) =
                    layer.advance_key_frame_data_bind_occurrence(id, graphs, elapsed_seconds)
                {
                    keep_going |= result;
                    break;
                }
            }
        }
        keep_going
    }
    pub(super) fn initialize_ordinary_data_bind_container(&mut self) {
        self.data_bind_container = RuntimeDataBindContainerQueue::default();
        self.data_bind_occurrences.clear();

        for (occurrence, data_bind_index) in self
            .data_bind_graph
            .add_data_binds_to_container(&mut self.data_bind_container)
        {
            debug_assert_eq!(occurrence, self.data_bind_occurrences.len());
            self.data_bind_occurrences
                .push(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index });
        }
    }
    pub(super) fn initialize_data_bind_container(&mut self) {
        self.initialize_ordinary_data_bind_container();
        self.append_scripted_data_binds_to_container();
    }
    /// cloneScriptedObject appends its binds after the ordinary StateMachine
    /// binds; it does not rebuild/re-home the ordinary container prefix.
    pub(super) fn append_scripted_data_binds_to_container(&mut self) {
        for (action_binding_index, binding) in self.scripted_object_bindings.iter_mut().enumerate()
        {
            for (occurrence, input_index) in
                binding.add_data_binds_to_container(&mut self.data_bind_container)
            {
                debug_assert_eq!(occurrence, self.data_bind_occurrences.len());
                self.data_bind_occurrences.push(
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    },
                );
            }
        }
    }
    pub(super) fn unbind_data_bind_source(&mut self, source_index: usize) {
        let Some(source) = self.data_bind_graph.sources.get_mut(source_index) else {
            return;
        };
        source.retained_bind.reset_preserving_notification();
        if let Some(converter) = source.converter.as_ref() {
            source
                .converter_data_binds
                .unbind(converter, &mut source.converter_state);
        }
        source.retained_structural_source = None;
        source.bound = false;
        source.reconcile_pending = false;
    }
    pub(super) fn unbind_data_binds(&mut self) {
        for source_index in 0..self.data_bind_graph.sources.len() {
            self.unbind_data_bind_source(source_index);
        }
        self.data_bind_graph.context_kind = RuntimeDataBindGraphContextKind::None;
        self.data_bind_graph.imported_view_model_context = None;
        self.data_bind_graph.default_view_model_bindings_dirty = false;
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            for source in &mut graph.sources {
                source.retained_bind.reset_preserving_notification();
                if let Some(converter) = source.converter.as_ref() {
                    source
                        .converter_data_binds
                        .unbind(converter, &mut source.converter_state);
                }
                source.retained_structural_source = None;
                source.bound = false;
                source.reconcile_pending = false;
            }
            graph.context_kind = RuntimeDataBindGraphContextKind::None;
            graph.imported_view_model_context = None;
            graph.default_view_model_bindings_dirty = false;
        }
    }
    pub(super) fn bind_owned_data_binds_from_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        #[cfg(test)]
        {
            self.owned_data_bind_context_bind_count += 1;
        }
        let mut changed = self
            .data_bind_graph
            .bind_owned_view_model_data_context_with_file(
                data_context,
                self.scripted_listener_runtime_file.as_deref(),
            );
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_data_context_with_file(
                data_context,
                self.scripted_listener_runtime_file.as_deref(),
            );
        }
        self.sync_bindable_font_assets_from_owned_data_context(data_context);
        changed
    }
    /// Mirrors C++ `DataBindContainer::updateDataBinds(false)`. Dirty
    /// source-to-target values must be visible to event listeners and
    /// transition conditions without polling or writing target-to-source
    /// bindings.
    pub(crate) fn update_data_binds_false(
        &mut self,
        artboard: &ArtboardInstance,
        owned_context: Option<&RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        // Retained cells cascade into per-bind sinks without borrowing this
        // machine. Fold that dirt before each applyEvents batch so listener-
        // authored chained writes have the same visibility as C++'s direct
        // `DataBind::addDirt` calls (`state_machine_instance.cpp:2328`).
        self.data_bind_graph.collect_retained_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.collect_retained_source_dirt();
        }
        let occurrences = self.data_bind_occurrences.clone();
        let queued = {
            let graph = &self.data_bind_graph;
            let scripted = &self.scripted_object_bindings;
            self.data_bind_container
                .begin_update(|occurrence| match occurrences.get(occurrence) {
                    Some(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index }) => {
                        graph.data_bind_is_to_source(*data_bind_index)
                    }
                    Some(RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    }) => scripted
                        .get(*action_binding_index)
                        .is_some_and(|binding| binding.data_bind_is_to_source(*input_index)),
                    None => false,
                })
        };
        let Some(queued) = queued else {
            return Ok(());
        };

        let file = artboard.runtime_file_arc();
        let resolver = self.scripted_listener_artboard_resolver.clone();
        let artboard_parent_context = self.scripted_listener_artboard_parent_context(None);
        let data_context = self.owned_data_context.clone();
        for (position, occurrence_index) in queued.iter().copied().enumerate() {
            self.data_bind_container.begin_occurrence(occurrence_index);
            let result = (|| -> Result<(), ScriptError> {
                let Some(occurrence) = occurrences.get(occurrence_index).copied() else {
                    return Ok(());
                };
                match occurrence {
                    RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index } => {
                        if let Some(file) = file.as_ref() {
                            let mut apply =
                                |instance: &RuntimeScriptInstanceHandle,
                                 input_name: &ScriptCoreString,
                                 value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                                 -> Result<(), ScriptError> {
                                    apply_scripted_input_update(
                                        instance,
                                        input_name,
                                        value,
                                        resolver.as_deref(),
                                        artboard_parent_context.as_ref(),
                                        host,
                                    )?;
                                    Ok(())
                                };
                            if let Some(data_context) = data_context.as_ref() {
                                self.data_bind_graph
                                    .update_converter_data_binds_from_data_context_for_data_bind(
                                        data_bind_index,
                                        file,
                                        data_context,
                                        &mut apply,
                                    )?;
                            } else if let Some(owned_context) = owned_context {
                                self.data_bind_graph
                                    .update_converter_data_binds_for_data_bind(
                                        data_bind_index,
                                        file,
                                        owned_context,
                                        &mut apply,
                                    )?;
                            }
                        }
                        if self.data_bind_graph.default_view_model_context_bound() {
                            self.update_default_view_model_binding(
                                data_bind_index,
                                true,
                                RuntimeDataBindGraphApplyPhase::UpdateDataBindsFalse,
                            )?;
                        }
                    }
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    } => {
                        let Some(file) = file.as_ref() else {
                            return Ok(());
                        };
                        let Some(binding) =
                            self.scripted_object_bindings.get_mut(action_binding_index)
                        else {
                            return Ok(());
                        };
                        let owner_instance = self
                            .scripted_listener_action_instances
                            .get(&binding.action_global_id())
                            .cloned();
                        let mut apply =
                            |instance: &RuntimeScriptInstanceHandle,
                             input_name: &ScriptCoreString,
                             value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                             -> Result<(), ScriptError> {
                                apply_scripted_input_update(
                                    instance,
                                    input_name,
                                    value,
                                    resolver.as_deref(),
                                    artboard_parent_context.as_ref(),
                                    host,
                                )?;
                                Ok(())
                            };
                        if let Some(update) = binding.public_update_data_bind(
                            input_index,
                            file,
                            owner_instance.as_ref(),
                            false,
                            &mut apply,
                        )? && let Some(instance) = self
                            .scripted_listener_action_instances
                            .get(&update.action_global_id)
                            .cloned()
                        {
                            apply_scripted_input_update(
                                &instance,
                                &update.input_name,
                                update.value,
                                resolver.as_deref(),
                                artboard_parent_context.as_ref(),
                                host,
                            )?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                self.data_bind_container
                    .abort_update(queued[position..].iter().copied());
                return Err(error);
            }
        }
        self.data_bind_container.finish_update();
        let _ = owned_context;
        Ok(())
    }
    pub(super) fn collect_retained_owned_view_model_dirt(&mut self) -> bool {
        // Retained property cells refresh through their individual dirt
        // sinks. Structural ViewModel replacement separately pushes the
        // parent relay's DataContext-rebind sink; no root generation is
        // sampled or compared on the steady frame.
        let (mut collected, mut schedules_advance) = self
            .data_bind_graph
            .collect_retained_source_dirt_with_schedule();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            let (graph_collected, graph_schedules_advance) =
                graph.collect_retained_source_dirt_with_schedule();
            collected |= graph_collected;
            schedules_advance |= graph_schedules_advance;
        }
        for binding in &mut self.scripted_object_bindings {
            let binding_collected = binding.collect_source_dirt();
            collected |= binding_collected;
            schedules_advance |= binding_collected;
        }
        if self.owned_data_context.is_none() {
            if schedules_advance {
                self.needs_advance = true;
            }
            return collected;
        }
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if structural_rebind {
            if self.primary_data_context.is_some() {
                self.refresh_primary_data_context_projection();
            }
            // The legacy ordinary/keyframe walk consumed the shared
            // structural sink, but fixed ScriptedObject occurrences have not
            // crossed their C++ rebind yet. Keep that work pending for the
            // facade's source-corresponding pre-operation pass.
            self.scripted_data_context_bind_complete = false;
            let data_context = self
                .owned_data_context
                .clone()
                .expect("owned context checked above");
            self.bind_owned_data_binds_from_data_context(&data_context);
            self.bind_view_model_listener_cells_for_data_context(&data_context);
            self.retain_owned_view_model_advance_context(&data_context);
        }
        if schedules_advance || structural_rebind {
            self.needs_advance = true;
        }
        collected || structural_rebind
    }
    pub fn update_data_binds_apply_target_to_source(&mut self) -> bool {
        let has_bound_data_binds = self.data_bind_graph.data_context_present()
            || !self.scripted_object_bindings.is_empty();
        let occurrences = self.data_bind_occurrences.clone();
        let queued = {
            let graph = &self.data_bind_graph;
            let scripted = &self.scripted_object_bindings;
            self.data_bind_container
                .begin_update(|occurrence| match occurrences.get(occurrence) {
                    Some(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index }) => {
                        graph.data_bind_is_to_source(*data_bind_index)
                    }
                    Some(RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    }) => scripted
                        .get(*action_binding_index)
                        .is_some_and(|binding| binding.data_bind_is_to_source(*input_index)),
                    None => false,
                })
        };
        let Some(queued) = queued else {
            return has_bound_data_binds;
        };

        let file = self.scripted_listener_runtime_file.clone();
        let resolver = self.scripted_listener_artboard_resolver.clone();
        let artboard_parent_context = self.scripted_listener_artboard_parent_context(None);
        let data_context = self.owned_data_context.clone();
        let mut host = NoopScriptHost;
        let mut changed = false;
        let mut result = Ok(());
        for (position, occurrence_index) in queued.iter().copied().enumerate() {
            self.data_bind_container.begin_occurrence(occurrence_index);
            let occurrence_result = (|| -> Result<(), ScriptError> {
                let Some(occurrence) = occurrences.get(occurrence_index).copied() else {
                    return Ok(());
                };
                match occurrence {
                    RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index } => {
                        if let (Some(file), Some(data_context)) =
                            (file.as_ref(), data_context.as_ref())
                        {
                            let mut apply =
                                |instance: &RuntimeScriptInstanceHandle,
                                 input_name: &ScriptCoreString,
                                 value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                                 -> Result<(), ScriptError> {
                                    changed |= apply_scripted_input_update(
                                        instance,
                                        input_name,
                                        value,
                                        resolver.as_deref(),
                                        artboard_parent_context.as_ref(),
                                        &mut host,
                                    )?;
                                    Ok(())
                                };
                            self.data_bind_graph
                                .update_converter_data_binds_from_data_context_for_data_bind(
                                    data_bind_index,
                                    file,
                                    data_context,
                                    &mut apply,
                                )?;
                        }
                        if self.data_bind_graph.default_view_model_context_bound() {
                            self.public_update_default_view_model_binding(data_bind_index)?;
                        }
                    }
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    } => {
                        let Some(file) = file.as_ref() else {
                            return Ok(());
                        };
                        let Some(binding) =
                            self.scripted_object_bindings.get_mut(action_binding_index)
                        else {
                            return Ok(());
                        };
                        let owner_instance = self
                            .scripted_listener_action_instances
                            .get(&binding.action_global_id())
                            .cloned();
                        let mut apply =
                            |instance: &RuntimeScriptInstanceHandle,
                             input_name: &ScriptCoreString,
                             value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                             -> Result<(), ScriptError> {
                                changed |= apply_scripted_input_update(
                                    instance,
                                    input_name,
                                    value,
                                    resolver.as_deref(),
                                    artboard_parent_context.as_ref(),
                                    &mut host,
                                )?;
                                Ok(())
                            };
                        if let Some(update) = binding.public_update_data_bind(
                            input_index,
                            file,
                            owner_instance.as_ref(),
                            true,
                            &mut apply,
                        )? && let Some(instance) = self
                            .scripted_listener_action_instances
                            .get(&update.action_global_id)
                            .cloned()
                        {
                            changed |= apply_scripted_input_update(
                                &instance,
                                &update.input_name,
                                update.value,
                                resolver.as_deref(),
                                artboard_parent_context.as_ref(),
                                &mut host,
                            )?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = occurrence_result {
                self.data_bind_container
                    .abort_update(queued[position..].iter().copied());
                result = Err(error);
                break;
            }
        }
        if result.is_ok() {
            self.data_bind_container.finish_update();
        }
        if let Err(error) = result {
            self.script_error = Some(error);
        }
        if changed {
            self.needs_advance = true;
        }
        has_bound_data_binds
    }
}
