// State-machine instance integration for the C++ `viewmodel_instance_trigger.cpp` source.
use super::*;
impl StateMachineInstance {
    pub fn set_bindable_trigger_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_trigger.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn bindable_trigger_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_trigger_value(&self.bindable_triggers, global_id))
        {
            return Some(value);
        }
        self.bindable_triggers
            .iter()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
            .map(|bindable_trigger| bindable_trigger.value)
    }
    fn set_default_view_model_trigger_cell_for_path(&mut self, path: &[u32], value: u64) -> bool {
        let Some(cell) = self
            .default_view_model_trigger_instance
            .as_ref()
            .and_then(|context| context.cell_for_source_path(path))
            .filter(|cell| matches!(cell.value(), RuntimeViewModelCellValue::Trigger(_)))
        else {
            return false;
        };
        if !cell.set_value(RuntimeViewModelCellValue::Trigger(value)) {
            return false;
        }
        self.data_bind_graph.collect_retained_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.collect_retained_source_dirt();
        }
        true
    }
    pub fn set_default_view_model_trigger_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .data_bind_graph
            .default_view_model_trigger_source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        if !self.set_default_view_model_trigger_cell_for_path(&path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_trigger_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_trigger_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self.set_default_view_model_trigger_cell_for_path(&path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn default_view_model_trigger_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path = runtime_default_view_model_trigger_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }
    pub fn default_view_model_trigger_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path =
            runtime_default_view_model_trigger_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }
    pub fn set_default_view_model_trigger_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        if !self.set_default_view_model_trigger_cell_for_path(&handle.path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_trigger_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub(super) fn fire_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let Some((&view_model_index, source_tail)) = source_path.split_first() else {
            return false;
        };
        if usize::try_from(view_model_index).ok() != Some(context.view_model_index) {
            return false;
        }
        let property_path = source_tail
            .iter()
            .map(|property_index| usize::try_from(*property_index).ok())
            .collect::<Option<Vec<_>>>();
        let Some(property_path) = property_path.filter(|path| !path.is_empty()) else {
            return false;
        };
        self.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
            context,
            data_bind_index,
            value,
            &property_path,
        )
    }
    pub(super) fn fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
        property_path: &[usize],
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        bindable_trigger.set_value(value);
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    context,
                    data_bind_index,
                    value,
                    property_path,
                )
            },
        );
        if !changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_trigger_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub(crate) fn reset_advanced_data_context(&mut self) {
        #[cfg(test)]
        {
            self.data_context_advance_call_count += 1;
        }
        if !self.data_bind_graph.default_view_model_context_bound() {
            return;
        }
        let file_instance_advanced = self.active_file_view_model_binding.is_some_and(
            |(view_model_index, instance_index)| {
                self.file_view_model_instances
                    .as_ref()
                    .is_some_and(|catalog| {
                        catalog.advance_instance(view_model_index, instance_index)
                    })
            },
        );
        if file_instance_advanced {
            // C++ `ViewModelInstance::advanced()` walks the retained values;
            // the catalog precomputes the root's unique nested/list trigger
            // cells so cyclic authored topology stays allocation-free here.
        } else if let Some(context) = &self.active_owned_view_model_advance_context {
            context.advanced();
        }
        let mut changed = self.data_bind_graph.collect_retained_trigger_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.collect_retained_trigger_source_dirt();
        }
        // Cloned ScriptInput DataBinds belong to this same C++
        // DataBindContainer. `ViewModelInstanceTrigger::advanced()` dirties
        // them synchronously even though delegation is suppressed; fold that
        // retained-cell notification now so the next bounded outer pass can
        // project the target back to zero (`viewmodel_instance_trigger.cpp:
        // 20-27`; `state_machine_instance.cpp:2629-2647`).
        for binding in &mut self.scripted_object_bindings {
            changed |= binding.collect_source_dirt();
        }
        if changed {
            self.needs_advance = true;
        }
    }
    pub fn view_model_trigger_count(&self, index: usize) -> Option<u64> {
        let trigger = self.default_view_model_triggers.get(index)?;
        let view_model_index = u32::try_from(self.default_view_model_index?).ok()?;
        let cell = self
            .default_view_model_trigger_instance
            .as_ref()?
            .cell_for_source_path(&[view_model_index, trigger.view_model_property_id])?;
        match cell.value() {
            RuntimeViewModelCellValue::Trigger(value) => Some(value),
            _ => None,
        }
    }
    /// #RB-1 e4 test seam: the retained cell a migrated listener condition is
    /// registered on, if any.
    #[cfg(test)]
    pub(crate) fn view_model_listener_condition_cell(
        &self,
        index: usize,
    ) -> Option<RuntimeViewModelCell> {
        self.view_model_listeners
            .get(index)?
            .property_bindings
            .first()?
            .cell_binding
            .as_ref()
            .map(|binding| binding.cell.clone())
    }
    pub fn view_model_trigger_value_count(&self) -> usize {
        self.default_view_model_triggers.len()
    }
    pub fn view_model_trigger_property_id(&self, index: usize) -> Option<u32> {
        self.default_view_model_triggers
            .get(index)
            .map(|trigger| trigger.view_model_property_id)
    }
}
