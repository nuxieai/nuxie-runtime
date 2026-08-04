// State-machine instance integration for the C++ `data_converter_group.cpp` source.
use super::*;

impl StateMachineInstance {
    /// Bind only one concrete converter occurrence, never its Group children.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn bind_scripted_listener_converter_own_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                if let Some(data_context) = data_context.as_ref() {
                    occurrence.bind_converter_own_sources_from_data_context_at_path(
                        file,
                        data_context,
                        input_global_id,
                        converter_path,
                        explicit_rebind,
                    )
                } else if let Some(root_context) = root_context {
                    occurrence.bind_converter_own_sources_at_path(
                        file,
                        root_context,
                        input_global_id,
                        converter_path,
                        explicit_rebind,
                    )
                } else {
                    false
                }
            })
    }
    #[doc(hidden)]
    pub fn scripted_listener_data_converter_targets(
        &self,
    ) -> Vec<(u32, u32, Vec<usize>, u32, bool)> {
        let mut targets = Vec::new();
        for occurrence in &self.scripted_object_bindings {
            for (input_global_id, converter_path, converter_global_id, inits) in
                occurrence.scripted_converter_targets()
            {
                targets.push((
                    occurrence.action_global_id(),
                    input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                ));
            }
        }
        targets
    }
    /// Enumerate every cloned ScriptedDataConverter occurrence, including
    /// those that already own a live script table.
    ///
    /// C++ calls `ScriptedDataConverter::reinit` on every DataContext bind,
    /// not only when the prior generator failed. The `attached` bit lets the
    /// facade distinguish generator work from persistent-table rehydration
    /// without aliasing occurrences by converter global id.
    #[doc(hidden)]
    pub fn scripted_listener_data_converter_occurrences(
        &self,
    ) -> Vec<(u32, u32, Vec<usize>, u32, bool, bool)> {
        let mut occurrences = Vec::new();
        for occurrence in &self.scripted_object_bindings {
            for (input_global_id, converter_path, converter_global_id, inits, attached) in
                occurrence.scripted_converter_occurrences()
            {
                occurrences.push((
                    occurrence.action_global_id(),
                    input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                    attached,
                ));
            }
        }
        occurrences
    }
    /// Immutable occurrence-keyed view of scripted converters cloned by the
    /// state machine's own authored DataBinds.
    ///
    /// This is a parity-evidence delegate, not a mutation or attachment API.
    /// A converter definition id may appear more than once; the parent
    /// DataBind index plus Group path is the concrete occurrence identity.
    #[doc(hidden)]
    pub fn scripted_data_converter_occurrence_snapshots(
        &self,
    ) -> Vec<crate::RuntimeScriptedDataConverterOccurrenceSnapshot> {
        self.data_bind_graph
            .scripted_converter_occurrence_snapshots()
    }
    #[doc(hidden)]
    pub fn state_machine_data_converter_bind_steps(
        &self,
    ) -> Vec<RuntimeStateMachineDataConverterBindStep> {
        runtime_state_machine_data_converter_bind_steps(&self.data_bind_graph)
    }
    #[doc(hidden)]
    pub fn scripted_data_converter_input_snapshots(
        &self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> Option<Vec<crate::ScriptListenerInputSnapshot>> {
        self.data_bind_graph
            .scripted_converter_input_snapshots_at_occurrence(
                parent_data_bind_index,
                converter_path,
            )
    }
    #[doc(hidden)]
    pub fn has_scripted_data_converter_instance(
        &self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> bool {
        self.data_bind_graph
            .scripted_converter_instance_at_occurrence(parent_data_bind_index, converter_path)
            .is_some()
    }
    #[doc(hidden)]
    pub fn bind_state_machine_data_bind_source(&mut self, data_bind_index: usize) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        self.data_bind_graph
            .bind_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context)
    }
    #[doc(hidden)]
    pub fn bind_state_machine_data_converter_own_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .bind_converter_own_sources_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                    explicit_rebind,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .bind_converter_own_sources_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                    explicit_rebind,
                )
        } else {
            false
        }
    }
    #[doc(hidden)]
    pub fn finalize_state_machine_data_bind_source(&mut self, data_bind_index: usize) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        self.data_bind_graph
            .finalize_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context)
    }
    #[doc(hidden)]
    pub fn rebind_state_machine_data_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .rebind_scripted_converter_final_input_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    input_index,
                    data_bind_index,
                    file,
                    data_context,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .rebind_scripted_converter_final_input_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    input_index,
                    data_bind_index,
                    file,
                    root_context,
                )
        } else {
            false
        }
    }
    /// Bind only the custom-input DataBinds owned by one concrete cloned
    /// `ScriptedDataConverter`.
    ///
    /// C++ performs this before `reinit`/hydration and repeats the direct
    /// ScriptInput bind after hydration. The parent DataBind index plus Group
    /// path identifies the clone; a converter definition id does not
    /// (`scripted_data_converter.cpp:170-188`;
    /// `data_converter_group.cpp:63-74`).
    #[doc(hidden)]
    pub fn bind_scripted_data_converter_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .bind_scripted_converter_sources_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                    explicit_rebind,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .bind_scripted_converter_sources_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                    explicit_rebind,
                )
        } else {
            false
        }
    }
    /// Repeat only each custom ScriptInput's retained final DataBind after a
    /// successful hydrate/init. Pinned C++ walks custom properties here, not
    /// the converter's complete DataBind collection
    /// (`scripted_data_converter.cpp:176-187`).
    #[doc(hidden)]
    pub fn rebind_scripted_data_converter_final_inputs(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .rebind_scripted_converter_final_inputs_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .rebind_scripted_converter_final_inputs_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                )
        } else {
            false
        }
    }
    #[doc(hidden)]
    pub fn set_scripted_data_converter_instance(
        &mut self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        converter_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        if !self.data_bind_graph.attach_scripted_instance_at_occurrence(
            parent_data_bind_index,
            converter_path,
            converter_global_id,
            &handle,
        ) {
            return Err(ScriptError::new(format!(
                "state-machine DataBind {parent_data_bind_index} has no ScriptedDataConverter occurrence {converter_path:?} (global {converter_global_id})",
            )));
        }
        Ok(())
    }
    /// Complete one C++ `ScriptedDataConverter::reinit` attempt for one
    /// state-machine DataBind occurrence.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_data_converter_instance<F>(
        &mut self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .data_bind_graph
            .scripted_converter_instance_at_occurrence(
                parent_data_bind_index,
                converter_path,
            )
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state-machine DataBind {parent_data_bind_index} has no attached ScriptedDataConverter occurrence {converter_path:?}",
                ))
            })?;
        let mut factory = factory;
        {
            let mut instance = handle.borrow_mut();
            context.install_context(&mut **instance)?;
            if let Some(factory) = factory.as_deref_mut() {
                instance.prepare_init_retry_with_factory(factory)?;
            } else {
                instance.prepare_init_retry()?;
            }
        }

        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        let hydrated = if !inits || !instance.user_init_pending()? {
            true
        } else if let Some(factory) = factory {
            instance.call_init_with_factory(&mut NoopScriptHost, factory)?
        } else {
            instance.call_init(&mut NoopScriptHost)?
        };
        drop(instance);
        if hydrated {
            let marked = self
                .data_bind_graph
                .mark_scripted_converter_hydrated_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                );
            debug_assert!(
                marked,
                "the hydrated ScriptedDataConverter must retain its exact parent DataBind"
            );
        }
        Ok(hydrated)
    }
    #[doc(hidden)]
    pub fn scripted_listener_data_converter_bind_steps(
        &self,
    ) -> Vec<super::RuntimeScriptedListenerDataConverterBindStep> {
        self.scripted_object_bindings
            .iter()
            .flat_map(|occurrence| occurrence.scripted_converter_bind_steps())
            .collect()
    }
    #[doc(hidden)]
    pub fn has_scripted_listener_data_converter_instance(
        &self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        self.scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                occurrence.has_scripted_converter_instance_at_path(input_global_id, converter_path)
            })
    }
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn rebind_scripted_listener_data_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        action_global_id: u32,
        listener_input_global_id: u32,
        converter_path: &[usize],
        converter_input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                occurrence.rebind_scripted_converter_final_input(
                    file,
                    root_context,
                    data_context.as_ref(),
                    listener_input_global_id,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                )
            })
    }
    #[doc(hidden)]
    pub fn scripted_listener_data_converter_input_snapshots(
        &self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> Option<Vec<crate::ScriptListenerInputSnapshot>> {
        self.scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)?
            .scripted_converter_input_snapshots(input_global_id, converter_path)
    }
    #[doc(hidden)]
    pub fn set_scripted_listener_data_converter_instance(
        &mut self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        converter_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        let Some(occurrence) = self
            .scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
        else {
            return Err(ScriptError::new(format!(
                "state machine has no scripted listener binding occurrence global {action_global_id}",
            )));
        };
        if !occurrence.attach_scripted_converter_instance_at_path(
            input_global_id,
            converter_path,
            &handle,
        ) {
            return Err(ScriptError::new(format!(
                "ScriptedListenerAction global {action_global_id} input global {input_global_id} has no ScriptedDataConverter occurrence {converter_path:?} (global {converter_global_id})",
            )));
        }
        Ok(())
    }
    /// Run one complete C++-ordered hydration attempt for one retained
    /// `ScriptedDataConverter` occurrence.
    ///
    /// `StateMachineInstance::internalDataContext` assigns the live context
    /// before `initScriptedObjects`; `ScriptedObject::hydrateScriptInputs`
    /// then validates the whole occurrence before applying any input, and
    /// only afterward calls user `init` and `didHydrateScriptInputs`
    /// (`state_machine_instance.cpp:2886-2913`;
    /// `scripted_object.cpp:313-437`). Keep those phases inside one API so a
    /// facade cannot repeat the context/generator preamble or accidentally
    /// validate multiple scripted-object occurrences as one transaction.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_listener_data_converter_instance<F>(
        &mut self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .and_then(|occurrence| {
                occurrence
                    .scripted_converter_instance_at_path(input_global_id, converter_path)
            })
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptedListenerAction global {action_global_id} input global {input_global_id} has no attached ScriptedDataConverter occurrence {converter_path:?}",
                ))
            })?;
        let mut factory = factory;

        {
            let mut instance = handle.borrow_mut();
            context.install_context(&mut **instance)?;
            if let Some(factory) = factory.as_deref_mut() {
                instance.prepare_init_retry_with_factory(factory)?;
            } else {
                instance.prepare_init_retry()?;
            }
        }

        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        let hydrated = if !inits || !instance.user_init_pending()? {
            true
        } else if let Some(factory) = factory {
            instance.call_init_with_factory(&mut NoopScriptHost, factory)?
        } else {
            instance.call_init(&mut NoopScriptHost)?
        };
        drop(instance);
        if hydrated {
            let marked = self
                .scripted_object_bindings
                .iter_mut()
                .find(|occurrence| occurrence.action_global_id() == action_global_id)
                .is_some_and(|occurrence| {
                    occurrence.mark_scripted_converter_hydrated(input_global_id, converter_path)
                });
            debug_assert!(
                marked,
                "the hydrated ScriptedDataConverter occurrence must retain its outer DataBind"
            );
        }
        Ok(hydrated)
    }
}
