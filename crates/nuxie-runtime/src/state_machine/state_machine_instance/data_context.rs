// State-machine instance integration for the C++ `data_context.cpp` source.
use super::state_machine_instance::{
    RuntimeViewModelListenerSource, listener_property_path_for_resolved_name_path,
    relink_view_model_listener_cell, resolved_listener_property_path_for_data_context,
    runtime_owned_font_asset_value_for_state_machine_source,
};
use super::*;
impl StateMachineInstance {
    /// Install one cloned `ScriptedObject`'s live DataContext without
    /// hydrating or initializing its script table.
    ///
    /// `StateMachineInstance::internalDataContext` assigns the new context to
    /// every retained ScriptedObject before `initScriptedObjects` enters the
    /// first occurrence (`state_machine_instance.cpp:2901-2913`). The facade
    /// uses this split operation to preserve that collection-wide barrier.
    #[doc(hidden)]
    pub fn install_scripted_object_data_context(
        &mut self,
        global_id: u32,
        context: &crate::ScriptListenerActionHydration,
    ) -> Result<(), ScriptError> {
        let handle = self
            .scripted_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "scripted object global {global_id} is not attached"
                ))
            })?;
        context.install_context(&mut **handle.borrow_mut())
    }
    /// Re-home the complete occurrence-owned DataContext onto a transaction's
    /// detached ViewModel roots without flattening its local/global/scoped/
    /// parent topology.
    #[doc(hidden)]
    pub fn rehome_owned_data_context_for_transaction(
        &mut self,
        roots: &[(RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelHandle)],
    ) {
        let Some(data_context) = self.owned_data_context.as_ref() else {
            return;
        };
        let data_context = data_context.rehomed_clone_with_roots(roots);
        self.owned_data_context = Some(data_context.clone());
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.retain_owned_view_model_advance_context(&data_context);
        self.register_owned_view_model_rebind_dependents();
        self.scripted_data_context_bind_complete = false;
    }
    /// Rust error projection for C++ pointer paths whose null behavior is not
    /// a safe clear/no-op. The C++-shaped methods below keep `Option` at the
    /// boundary so their intentionally different null branches cannot be
    /// collapsed by a typed convenience API.
    #[doc(hidden)]
    pub(crate) fn bind_data_context(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_context = data_context.ok_or(RuntimeDataContextBindError::NullDataContext)?;
        // Pinned C++: clear the machine registration, register the supplied
        // context, clear/bind the artboard, then bind the machine.
        self.clear_data_context();
        self.primary_data_context = Some(data_context.clone());
        self.record_bind_phase("register-machine");
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        let projection = data_context.projection();
        self.record_bind_phase("clear-artboard");
        artboard.clear_data_context_for_state_machine_bind();
        self.record_bind_phase("bind-artboard");
        let mut changed =
            artboard.bind_owned_view_model_artboard_data_context(file, &projection, true, true);
        data_context.add_artboard_rebind_dependent(artboard);
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(Some(&projection))?;
        Ok(changed)
    }
    /// C++ `inheritDataContext`: null is a no-op and, critically, the old
    /// context is not cleared before the new one registers this same sink.
    /// A→B therefore leaves a live weak registration on A while the retained
    /// context pointer and all paths refer to B.
    #[doc(hidden)]
    pub(crate) fn inherit_data_context(
        &mut self,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let Some(data_context) = data_context else {
            return Ok(false);
        };
        self.primary_data_context = Some(data_context.clone());
        self.record_bind_phase("register-machine-without-clear");
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.internal_data_context(Some(&data_context.projection()))
    }
    /// C++ `dataContext(rcp<DataContext>)`: clear only the machine
    /// registration/listener cells, then forward the supplied pointer to the
    /// internal binder without registering it or touching the artboard.
    #[doc(hidden)]
    pub(crate) fn set_data_context(
        &mut self,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.clear_data_context();
        self.primary_data_context = data_context.cloned();
        let projection = data_context.map(RuntimeStateMachineDataContext::projection);
        self.internal_data_context(projection.as_ref())
    }
    /// Borrowed counterpart of C++ `dataContext() const`.
    #[doc(hidden)]
    pub(crate) fn data_context(&self) -> Option<&RuntimeStateMachineDataContext> {
        self.primary_data_context.as_ref()
    }
    /// C++ `setViewModelInstance`: a null pointer is an inert no-op; a live
    /// instance replaces only the main slot and does not bind any path.
    #[doc(hidden)]
    pub(crate) fn set_view_model_instance(
        &mut self,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(view_model_instance) = view_model_instance else {
            return false;
        };
        let context = self.ensure_primary_data_context();
        context.set_main(view_model_instance);
        true
    }
    /// C++ `setGlobalViewModelInstance`: validate the named file slot, then
    /// replace or empty exactly that slot. The occupying instance may belong
    /// to a different ViewModel; slot identity comes from `name`.
    #[doc(hidden)]
    pub fn set_global_view_model_instance(
        &mut self,
        file: Option<&RuntimeFile>,
        name: &str,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(file) = file else {
            return false;
        };
        let mut validated_slot = RuntimeOwnedViewModelContext::default();
        let valid = match view_model_instance.as_ref() {
            Some(instance) => validated_slot.set_global_named_handle(file, name, instance.clone()),
            None => validated_slot.unset_global_named(file, name),
        };
        if !valid {
            return false;
        }
        if view_model_instance.is_none() && self.primary_data_context.is_none() {
            return true;
        }
        let context = self.ensure_primary_data_context();
        let changed = view_model_instance.map_or_else(
            || context.unset_global_named(file, name),
            |instance| context.set_global_named(file, name, instance),
        );
        if !changed {
            return false;
        }
        true
    }
    /// Fill the missing main first, then missing globals in file-global
    /// order. `RuntimeOwnedViewModelContext` stores globals in slot-key order
    /// and treats any existing cross-model occupant as occupied.
    #[doc(hidden)]
    pub(crate) fn complete_view_model_instances(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &ArtboardInstance,
    ) -> bool {
        let (Some(file), Some(context)) = (file, self.primary_data_context.clone()) else {
            return false;
        };
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|candidate| candidate.id == artboard.graph_global_id)
        else {
            return false;
        };
        if !context.complete_for_artboard(file, artboard_index) {
            return false;
        }
        true
    }
    /// C++ `bind`: create an empty retained context when needed, complete
    /// missing defaults, bind the artboard, then bind this machine.
    #[doc(hidden)]
    pub(crate) fn bind(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &mut ArtboardInstance,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.ensure_primary_data_context();
        self.record_bind_phase("complete-view-models");
        self.complete_view_model_instances(file, artboard);
        let data_context = self
            .primary_data_context
            .clone()
            .expect("checked retained DataContext")
            .projection();
        self.record_bind_phase("bind-artboard");
        let mut changed = file.is_some_and(|file| {
            artboard.bind_owned_view_model_artboard_data_context(file, &data_context, true, true)
        });
        if file.is_some()
            && let Some(context) = self.primary_data_context.as_ref()
        {
            context.add_artboard_rebind_dependent(artboard);
        }
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(Some(&data_context))?;
        Ok(changed)
    }
    #[doc(hidden)]
    pub fn bind_for_command_queue(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        self.bind(file, artboard).is_ok()
    }
    #[doc(hidden)]
    pub fn testing_main_view_model_is(&self, expected: &RuntimeOwnedViewModelHandle) -> bool {
        self.primary_data_context
            .as_ref()
            .and_then(RuntimeStateMachineDataContext::main_handle)
            .is_some_and(|actual| actual.ptr_eq(expected))
    }
    /// Convenience C++ member with deliberately asymmetric null behavior.
    /// Null clears only the machine context/listener cells and unbinds the
    /// artboard. It must not explicitly unbind this machine's DataBinds.
    #[doc(hidden)]
    pub(crate) fn bind_view_model_instance(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &mut ArtboardInstance,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let Some(view_model_instance) = view_model_instance else {
            self.clear_data_context();
            self.record_bind_phase("unbind-artboard");
            artboard.unbind_for_state_machine_view_model_clear(file);
            return Ok(true);
        };
        self.set_view_model_instance(Some(view_model_instance));
        self.bind(file, artboard)
    }
    /// Pure C++ slot read. Unlike the setter, the lookup intentionally does
    /// not reject a non-global name before consulting that numeric slot.
    #[doc(hidden)]
    pub fn global_view_model_instance(
        &self,
        file: Option<&RuntimeFile>,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelHandle> {
        let file = file?;
        let slot = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))?;
        self.primary_data_context.as_ref()?.global_slot_handle(slot)
    }
    /// C++ `rebind`: clear/reapply the artboard first, then reapply the exact
    /// retained machine context. A cleared/null context is still forwarded to
    /// the internal paths and can therefore fail at a ViewModel listener.
    #[doc(hidden)]
    pub(crate) fn rebind(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_context = self
            .primary_data_context
            .as_ref()
            .map(RuntimeStateMachineDataContext::projection);
        self.record_bind_phase("clear-artboard");
        artboard.clear_data_context_for_state_machine_bind();
        self.record_bind_phase("bind-artboard");
        let mut changed = data_context.as_ref().is_some_and(|data_context| {
            artboard.bind_owned_view_model_artboard_data_context(file, data_context, true, true)
        });
        if data_context.is_some()
            && let Some(context) = self.primary_data_context.as_ref()
        {
            context.add_artboard_rebind_dependent(artboard);
        }
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(data_context.as_ref())?;
        Ok(changed)
    }
    /// C++ `clearDataContext`: unregister/null first, then drop listener
    /// property cells. It does not unbind state-machine DataBinds or touch the
    /// artboard/script occurrences.
    #[doc(hidden)]
    pub(crate) fn clear_data_context(&mut self) {
        self.record_bind_phase("clear-machine");
        self.primary_data_context = None;
        self.owned_data_context = None;
        self.active_owned_view_model_advance_context = None;
        self.active_file_view_model_binding = None;
        self.scripted_data_context_bind_complete = false;
        // Dropping this sink makes all old weak registrations inert, the Rust
        // equivalent of removeDependentContainer.
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.clear_view_model_listener_cell_bindings();
    }
    /// C++ delegates this member exclusively to the artboard.
    #[doc(hidden)]
    pub(crate) fn relink_data_context(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        artboard.relink_data_context_for_state_machine(file)
    }
    /// Rebuild only one context-bind subtype. A plain authored DataBind is
    /// ignored; a null pointer is an error, matching the C++ dereference.
    #[doc(hidden)]
    pub(crate) fn rebuild_data_bind(
        &mut self,
        data_bind_index: Option<usize>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_bind_index = data_bind_index.ok_or(RuntimeDataContextBindError::NullDataBind)?;
        let Some(source_index) = self
            .data_bind_graph
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source.0)
        else {
            return Ok(false);
        };
        if !self
            .data_bind_graph
            .sources
            .get(source_index)
            .is_some_and(|source| source.context_bindable)
        {
            return Ok(false);
        }
        let Some(data_context) = self.owned_data_context.clone() else {
            self.unbind_data_bind_source(source_index);
            return Ok(false);
        };
        let mut changed = self
            .data_bind_graph
            .bind_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context);
        changed |= self
            .data_bind_graph
            .finalize_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context);
        if changed {
            self.needs_advance = true;
        }
        Ok(changed)
    }
    /// C++ `unbind`: context/listener teardown precedes every machine
    /// DataBind source/converter unbind.
    #[doc(hidden)]
    pub(crate) fn unbind(&mut self) {
        self.clear_data_context();
        self.unbind_data_binds();
    }
    /// C++ `internalDataContext` primary machine path: assign, bind ordinary
    /// and keyframe DataBinds, bind listener cells, then hand the new context
    /// to the deferred scripted-object context/init passes.
    #[doc(hidden)]
    pub(crate) fn internal_data_context(
        &mut self,
        data_context: Option<&RuntimeOwnedDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.record_bind_phase("assign-context");
        self.owned_data_context = data_context.cloned();
        let Some(data_context) = data_context else {
            self.record_bind_phase("bind-data-binds");
            self.unbind_data_binds();
            self.record_bind_phase("bind-listener-cells");
            self.clear_view_model_listener_cell_bindings();
            if !self.view_model_listeners.is_empty() {
                return Err(RuntimeDataContextBindError::NullDataContextWithViewModelListeners);
            }
            self.record_bind_phase("script-context-pass");
            self.scripted_data_context_bind_complete = false;
            self.record_bind_phase("script-init-pass");
            self.active_owned_view_model_advance_context = None;
            return Ok(true);
        };

        self.record_bind_phase("bind-data-binds");
        let changed = self.bind_owned_data_binds_from_data_context(data_context);
        self.record_bind_phase("bind-listener-cells");
        self.bind_view_model_listener_cells_for_data_context(data_context);
        // Rust's authenticated scripting facade owns the fallible table
        // context/install and init/hydrate calls. Mark that exact later pass
        // only after every listener cell has been rebound.
        self.record_bind_phase("script-context-pass");
        self.scripted_data_context_bind_complete = false;
        self.record_bind_phase("script-init-pass");
        self.retain_owned_view_model_advance_context(data_context);
        self.needs_advance = true;
        Ok(changed)
    }
    fn ensure_primary_data_context(&mut self) -> RuntimeStateMachineDataContext {
        if let Some(context) = self.primary_data_context.clone() {
            // Reusing an existing DataContext must preserve its registration
            // status. In particular, `dataContext(value)` intentionally
            // installs without `addDependentContainer`.
            return context;
        }
        let context = RuntimeStateMachineDataContext::default();
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(context.clone());
        context
    }
    pub(super) fn refresh_primary_data_context_projection(&mut self) {
        let Some(context) = self.primary_data_context.as_ref() else {
            self.owned_data_context = None;
            return;
        };
        let data_context = context.projection();
        self.active_file_view_model_binding = None;
        self.owned_data_context = Some(data_context.clone());
        self.retain_owned_view_model_advance_context(&data_context);
    }
    /// Machine-only borrow-model adaptation used by the established typed
    /// Rust APIs, which do not own an Artboard borrow in their signatures.
    fn bind_data_context_to_machine(&mut self, data_context: &RuntimeOwnedDataContext) -> bool {
        self.clear_data_context();
        self.primary_data_context = None;
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.internal_data_context(Some(data_context))
            .unwrap_or(false)
    }
    /// Preserve the established typed Rust context representations while
    /// making their public entry points pure delegating adaptations. The
    /// C++-shaped clear member owns replacement teardown; the closure contains
    /// only the representation-specific graph/listener projection required to
    /// preserve each existing boolean API's signature and behavior.
    fn bind_typed_context_adaptation(
        &mut self,
        bind: impl FnOnce(&mut StateMachineInstance) -> bool,
    ) -> bool {
        self.clear_data_context();
        bind(self)
    }
    pub fn bind_empty_data_context(&mut self) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            if !machine.data_bind_graph.bind_empty_data_context() {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_empty_data_context();
            }
            machine.active_file_view_model_binding = None;
            machine.needs_advance = true;
            true
        })
    }
    pub fn bind_default_view_model_context(&mut self) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            if !machine.data_bind_graph.bind_default_view_model_context() {
                return false;
            }
            if let Some(context) = machine.default_view_model_trigger_instance.as_ref() {
                machine
                    .data_bind_graph
                    .bind_file_view_model_trigger_sources(context);
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_default_view_model_context();
                if let Some(context) = machine.default_view_model_trigger_instance.as_ref() {
                    graph.bind_file_view_model_trigger_sources(context);
                }
            }
            machine.sync_bindable_font_assets_from_default_context();
            machine.active_file_view_model_binding =
                machine.default_view_model_index.map(|index| (index, 0));
            machine.needs_advance = true;
            true
        })
    }
    /// Create and bind the artboard's authored default `DataContext`.
    ///
    /// This is the C++ `createDefaultViewModelInstance(artboard)` followed by
    /// `StateMachineInstance::bindViewModelInstance` path. Unlike the
    /// graph-only compatibility method above, the mutable `DataContext` is
    /// shared with the artboard tree, so nested artboards and the outer state
    /// machine observe the same retained ViewModel cells.
    pub fn bind_default_view_model_context_on_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        let Some(file) = artboard.runtime_file_arc() else {
            return false;
        };
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|candidate| candidate.id == artboard.graph_global_id)
        else {
            return false;
        };
        let context = RuntimeStateMachineDataContext::default();
        if !context.complete_for_artboard(&file, artboard_index) {
            return false;
        }
        self.bind_data_context(&file, artboard, Some(&context))
            .unwrap_or(false)
    }
    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn debug_set_bound_main_font_bytes_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        font_bytes: Option<std::sync::Arc<[u8]>>,
    ) -> bool {
        let Some(data_context) = self.owned_data_context.as_ref() else {
            return false;
        };
        let Some(main) = data_context.main_context_chain(file).into_iter().next() else {
            return false;
        };
        if !main.scope_path().is_empty() {
            return false;
        }
        let changed = main
            .root_handle()
            .borrow_mut()
            .set_live_font_bytes_by_property_name(property_name, font_bytes);
        if changed {
            self.needs_advance = true;
        }
        changed
    }
    pub fn bind_view_model_instance_context(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let Some(instance_cells) = machine
                .file_view_model_instances
                .as_ref()
                .and_then(|catalog| catalog.instance(view_model_index, instance_index))
            else {
                return false;
            };
            if !machine.data_bind_graph.bind_view_model_instance_context(
                file,
                view_model_index,
                instance_index,
                &instance_cells,
            ) {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_view_model_instance_context(
                    file,
                    view_model_index,
                    instance_index,
                    &instance_cells,
                );
            }
            machine.sync_bindable_font_assets_from_imported_instance(
                file,
                view_model_index,
                instance_index,
            );
            machine.active_file_view_model_binding = Some((view_model_index, instance_index));
            machine.needs_advance = true;
            true
        })
    }
    pub fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let Some(instance) = machine
                .file_view_model_instances
                .as_ref()
                .and_then(|catalog| {
                    catalog.instance(context.view_model_index, context.instance_index)
                })
            else {
                return false;
            };
            if !context.adopt_file_trigger_instance(instance) {
                return false;
            }
            if !machine
                .data_bind_graph
                .bind_imported_view_model_context(file, context)
            {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_imported_view_model_context(file, context);
            }
            machine.sync_bindable_font_assets_from_imported_instance(
                file,
                context.view_model_index,
                context.instance_index,
            );
            machine.bind_view_model_listener_cells_for_imported_context(context);
            machine.active_file_view_model_binding =
                Some((context.view_model_index, context.instance_index));
            machine.needs_advance = true;
            true
        })
    }
    /// Snapshot an owned ViewModel context into this machine.
    ///
    /// ViewModel listeners dispatch their ordinary input/event actions, but an
    /// immutable borrow cannot receive listener-authored ViewModel writes. Use
    /// [`Self::bind_owned_view_model_context_mut`] or the owning artboard's
    /// context-aware advance API when those writes must update the host context.
    pub fn bind_owned_view_model_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            machine.bind_owned_view_model_snapshot(context)
        })
    }
    /// Bind and retain a shared owned view-model graph.
    ///
    /// Later mutations through any alias are refreshed at the next data
    /// context advance, so the state machine and host never fork identity.
    pub fn bind_owned_view_model_handle(&mut self, context: &RuntimeOwnedViewModelHandle) -> bool {
        let staged = RuntimeOwnedViewModelContext::from_main_handle(context.clone());
        let context = RuntimeOwnedViewModelContextHandle::root_without_file(context.clone());
        let changed = self.bind_owned_view_model_context_handle(&context);
        let primary = RuntimeStateMachineDataContext::from_owned_context(staged);
        // The immutable adaptation registered this sink directly on the
        // current root. Rotate it before installing the mutable carrier so a
        // later setMain makes that old weak registration inert.
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        primary.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(primary);
        changed
    }
    pub fn bind_owned_view_model_context_handle(
        &mut self,
        context: &RuntimeOwnedViewModelContextHandle,
    ) -> bool {
        self.bind_owned_view_model_data_context(&RuntimeOwnedDataContext::from_context_handle(
            context,
        ))
    }
    /// Install a facade-supplied live context without eagerly walking any
    /// DataBind. The scripting facade then executes the one C++-ordered
    /// ordinary + cloned-ScriptedObject container before calling
    /// [`Self::finish_scripted_object_data_context_bind`].
    #[doc(hidden)]
    pub fn begin_scripted_object_data_context_bind(
        &mut self,
        context: &RuntimeOwnedViewModelHandle,
    ) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_facade_root_requires_rebind(Some(context)) {
            self.require_scripted_object_data_context_rebind();
        }
        self.active_file_view_model_binding = None;
        let data_context = self
            .owned_data_context
            .as_ref()
            .filter(|bound| bound.main_root_matches(context))
            .cloned()
            .unwrap_or_else(|| {
                let context_handle =
                    RuntimeOwnedViewModelContextHandle::root_without_file(context.clone());
                RuntimeOwnedDataContext::from_context_handle(&context_handle)
            });
        let identity_changed = self
            .owned_data_context
            .as_ref()
            .is_none_or(|bound| !bound.same_binding(&data_context));
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if !identity_changed && !structural_rebind && self.scripted_data_context_bind_complete {
            return false;
        }
        // C++ `dataContext()` clears every ListenerViewModel registration
        // before `internalDataContext()` enters the first DataBind/converter
        // callback (`state_machine_instance.cpp:2880-2913,2923-2933`).
        // Leaving the old cells attached until `finish_*` would let a
        // converter callback enqueue a report through the previous context.
        self.clear_view_model_listener_cell_bindings();
        self.owned_data_context = Some(data_context);
        self.scripted_data_context_bind_complete = false;
        if identity_changed {
            self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        }
        self.needs_advance = true;
        true
    }
    /// Begin a C++ `rebind()` against the exact retained DataContext.
    ///
    /// This is the no-root facade path: it must preserve authored local,
    /// global, scoped, and parent instances rather than rebuilding a
    /// main-only context from a convenience argument.
    #[doc(hidden)]
    pub fn begin_retained_scripted_object_data_context_rebind(&mut self) -> bool {
        if self.script_error.is_some() || self.owned_data_context.is_none() {
            return false;
        }
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if !structural_rebind && self.scripted_data_context_bind_complete {
            return false;
        }
        self.clear_view_model_listener_cell_bindings();
        self.scripted_data_context_bind_complete = false;
        self.needs_advance = true;
        true
    }
    /// Complete C++ `StateMachineInstance::internalDataContext` after the
    /// facade has walked every outer DataBind and converter occurrence.
    #[doc(hidden)]
    pub fn finish_scripted_object_data_context_bind(&mut self) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        #[cfg(test)]
        {
            self.owned_data_bind_context_bind_count += 1;
        }
        self.sync_bindable_font_assets_from_owned_data_context(&data_context);
        self.bind_view_model_listener_cells_for_data_context(&data_context);
        self.retain_owned_view_model_advance_context(&data_context);
        self.register_owned_view_model_rebind_dependents();
        self.scripted_data_context_bind_complete = true;
        self.needs_advance = true;
        true
    }
    #[doc(hidden)]
    pub fn bind_script_artboard_data_context(
        &mut self,
        context: &ScriptArtboardDataContext,
    ) -> bool {
        self.bind_owned_view_model_data_context(context.runtime_context())
    }
    fn bind_owned_view_model_snapshot(&mut self, context: &RuntimeOwnedViewModelInstance) -> bool {
        self.active_file_view_model_binding = None;
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        advance_context.extend(context);
        self.active_owned_view_model_advance_context = Some(advance_context);
        let mut changed = self.data_bind_graph.bind_owned_view_model_context(context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context(context);
        }
        self.sync_bindable_font_assets_from_owned_context(context);
        self.bind_view_model_listener_cells_for_context_chain(context, &[&[]]);
        if changed {
            self.needs_advance = true;
        }
        changed
    }
    /// Rebind an owned ViewModel context. Typed ViewModel-change listeners
    /// retain their condition cells here and dispatch at next-frame start.
    pub fn bind_owned_view_model_context_mut(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
            advance_context.extend(context);
            machine.active_owned_view_model_advance_context = Some(advance_context);
            let mut changed = machine
                .data_bind_graph
                .bind_owned_view_model_context(context);
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                changed |= graph.bind_owned_view_model_context(context);
            }
            machine.sync_bindable_font_assets_from_owned_context(context);
            machine.bind_view_model_listener_cells_for_context_chain(context, &[&[]]);
            if changed {
                machine.needs_advance = true;
            }
            changed
        })
    }
    fn bind_view_model_listener_cells_for_context_chain(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) {
        let runtime_file = self.scripted_listener_runtime_file.as_deref();
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let definition = &listener.listener_definitions[listener.listener_index];
            for binding in &mut listener.property_bindings {
                let path = match binding.source {
                    RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                    RuntimeViewModelListenerSource::Input(input_index) => definition
                        .view_model_input_types
                        .get(input_index)
                        .and_then(|input| input.path()),
                };
                let cell = path.and_then(|path| match path {
                    RuntimeListenerViewModelPath::Absolute {
                        view_model_index,
                        property_path,
                    } => context_chain.iter().find_map(|context_path| {
                        context.cell_by_scoped_property_path(
                            context_path,
                            *view_model_index,
                            property_path,
                        )
                    }),
                    RuntimeListenerViewModelPath::Relative {
                        resolved_name_ids,
                        absolute_fallback,
                    } => {
                        let file = runtime_file?;
                        if file.manifest().is_some() {
                            context_chain.iter().find_map(|context_path| {
                                let property_path = listener_property_path_for_resolved_name_path(
                                    context,
                                    file,
                                    context_path,
                                    resolved_name_ids,
                                )?;
                                context.cell_by_property_path(&property_path)
                            })
                        } else {
                            let (view_model_index, property_path) = absolute_fallback.as_ref()?;
                            context_chain.iter().find_map(|context_path| {
                                context.cell_by_scoped_property_path(
                                    context_path,
                                    *view_model_index,
                                    property_path,
                                )
                            })
                        }
                    }
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
        }
    }
    pub(super) fn bind_view_model_listener_cells_for_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) {
        let runtime_file = self.scripted_listener_runtime_file.as_deref();
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let definition = &listener.listener_definitions[listener.listener_index];
            for binding in &mut listener.property_bindings {
                let path = match binding.source {
                    RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                    RuntimeViewModelListenerSource::Input(input_index) => definition
                        .view_model_input_types
                        .get(input_index)
                        .and_then(|input| input.path()),
                };
                let cell = path.and_then(|path| {
                    let resolved = match path {
                        RuntimeListenerViewModelPath::Absolute {
                            view_model_index,
                            property_path,
                        } => {
                            let mut source_path = Vec::with_capacity(property_path.len() + 1);
                            source_path.push(u32::try_from(*view_model_index).ok()?);
                            source_path.extend(
                                property_path
                                    .iter()
                                    .copied()
                                    .map(u32::try_from)
                                    .collect::<Result<Vec<_>, _>>()
                                    .ok()?,
                            );
                            data_context.resolved_property_path(&source_path)
                        }
                        RuntimeListenerViewModelPath::Relative {
                            resolved_name_ids,
                            absolute_fallback,
                        } => {
                            let file = runtime_file?;
                            if file.manifest().is_some() {
                                resolved_listener_property_path_for_data_context(
                                    data_context,
                                    file,
                                    resolved_name_ids,
                                )
                            } else {
                                let (view_model_index, property_path) =
                                    absolute_fallback.as_ref()?;
                                let mut source_path = Vec::with_capacity(property_path.len() + 1);
                                source_path.push(u32::try_from(*view_model_index).ok()?);
                                source_path.extend(
                                    property_path
                                        .iter()
                                        .copied()
                                        .map(u32::try_from)
                                        .collect::<Result<Vec<_>, _>>()
                                        .ok()?,
                                );
                                data_context.resolved_property_path(&source_path)
                            }
                        }
                    };
                    // C++ `DataContext::getViewModelProperty` returns the
                    // retained `ViewModelInstanceValue`; every authored
                    // ListenerInputTypeViewModel registers its own binding
                    // against the same parent ListenerViewModel
                    // (`state_machine_instance.cpp:1349-1372,1401-1407`).
                    resolved.and_then(|(context, property_path)| {
                        context.borrow().cell_by_property_path(&property_path)
                    })
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
        }
    }
    fn bind_view_model_listener_cells_for_imported_context(
        &mut self,
        context: &RuntimeImportedViewModelInstanceContext,
    ) {
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let definition = &listener.listener_definitions[listener.listener_index];
            for binding in &mut listener.property_bindings {
                let path = match binding.source {
                    RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                    RuntimeViewModelListenerSource::Input(input_index) => definition
                        .view_model_input_types
                        .get(input_index)
                        .and_then(|input| input.path()),
                };
                let cell = path.and_then(|path| {
                    let (view_model_index, property_path) = match path {
                        RuntimeListenerViewModelPath::Absolute {
                            view_model_index,
                            property_path,
                        } => (*view_model_index, property_path.as_slice()),
                        RuntimeListenerViewModelPath::Relative {
                            absolute_fallback: Some((view_model_index, property_path)),
                            ..
                        } => (*view_model_index, property_path.as_slice()),
                        RuntimeListenerViewModelPath::Relative {
                            absolute_fallback: None,
                            ..
                        } => return None,
                    };
                    if view_model_index != context.view_model_index {
                        return None;
                    }
                    let mut source_path = Vec::with_capacity(property_path.len() + 1);
                    source_path.push(u32::try_from(view_model_index).ok()?);
                    source_path.extend(
                        property_path
                            .iter()
                            .copied()
                            .map(u32::try_from)
                            .collect::<Result<Vec<_>, _>>()
                            .ok()?,
                    );
                    context.trigger_cell_for_source_path(&source_path)
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
        }
    }
    pub(super) fn perform_listener_view_model_change_for_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let asset_value =
            matches!(value, RuntimeListenerViewModelChangeValue::Asset(_)).then(|| {
                let RuntimeListenerViewModelChangeValue::Asset(fallback) = value else {
                    unreachable!("asset listener value was checked above")
                };
                self.listener_asset_value_for_data_bind(data_bind_index, fallback)
                    .clone()
            });

        let Some((context, property_path)) = data_context.resolved_property_path(&source_path)
        else {
            return false;
        };
        let mut context = context.borrow_mut();
        let changed = match value {
            RuntimeListenerViewModelChangeValue::Trigger(value) => Some(
                self.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    &mut context,
                    data_bind_index,
                    *value,
                    &property_path,
                ),
            ),
            _ => Self::apply_listener_view_model_change_at_property_path(
                &mut context,
                &property_path,
                value,
                asset_value.as_ref(),
            ),
        };
        changed.unwrap_or(false)
    }
    #[cfg(test)]
    pub(crate) fn owned_data_bind_context_bind_count(&self) -> usize {
        self.owned_data_bind_context_bind_count
    }
    pub(super) fn perform_listener_actions_for_data_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        data_context: &RuntimeOwnedDataContext,
        listener_actions: &[RuntimeScheduledListenerAction],
        invocation: &ScriptListenerInvocation,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        for action in listener_actions {
            if let RuntimeScheduledListenerAction::ViewModelChange(action) = action
                && let Some(bindable_global_id) = action.bindable_global_id
            {
                let value = {
                    let targets = RuntimeScheduledListenerActionTargetsMut {
                        inputs: &mut self.inputs,
                        reported_events: &mut self.reported_events,
                        bindable_numbers: &mut self.bindable_numbers,
                        bindable_integers: &mut self.bindable_integers,
                        bindable_colors: &mut self.bindable_colors,
                        bindable_strings: &mut self.bindable_strings,
                        bindable_enums: &mut self.bindable_enums,
                        bindable_assets: &mut self.bindable_assets,
                        bindable_artboards: &mut self.bindable_artboards,
                        bindable_lists: &mut self.bindable_lists,
                        bindable_triggers: &mut self.bindable_triggers,
                        bindable_view_models: &mut self.bindable_view_models,
                        bindable_booleans: &mut self.bindable_booleans,
                        transition_durations: &mut self.transition_durations,
                    };
                    action.occurrence_value(&targets, true)
                };
                let Some(value) = value else {
                    continue;
                };
                let source_changed = self
                    .data_bind_graph
                    .bindable_data_bind_to_source_index(bindable_global_id)
                    .is_some_and(|data_bind_index| {
                        self.perform_listener_view_model_change_for_data_context(
                            data_context,
                            data_bind_index,
                            &value,
                        )
                    });
                let target_dirtied = self
                    .data_bind_graph
                    .dirty_bindable_data_bind_to_target(bindable_global_id);
                changed |= source_changed || target_dirtied;
                // The retained source cell carries this mutation into the
                // next `updateDataBinds(false)` batch. Rebinding every source
                // here would spuriously reconcile unrelated two-way binds.
                continue;
            }
            changed |= self.perform_listener_actions(
                artboard,
                std::slice::from_ref(action),
                None,
                invocation,
                &mut NoopScriptHost,
            )?;
        }
        Ok(changed)
    }
    pub fn bind_owned_view_model_contexts(
        &mut self,
        context: &RuntimeOwnedViewModelContext,
    ) -> bool {
        let changed = self.bind_owned_view_model_data_context(
            &RuntimeOwnedDataContext::from_owned_context(context),
        );
        let primary = RuntimeStateMachineDataContext::from_owned_context(context.clone());
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        primary.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(primary);
        changed
    }
    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
            advance_context.extend(context);
            machine.active_owned_view_model_advance_context = Some(advance_context);
            let mut changed = machine.data_bind_graph.bind_owned_view_model_context_chain(
                file,
                context,
                context_chain,
            );
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                changed |= graph.bind_owned_view_model_context_chain(file, context, context_chain);
            }
            machine.sync_bindable_font_assets_from_owned_context_chain(
                file,
                context,
                context_chain,
            );
            machine.bind_view_model_listener_cells_for_context_chain(context, context_chain);
            if changed {
                machine.needs_advance = true;
            }
            changed
        })
    }
    pub(crate) fn bind_owned_view_model_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        self.bind_data_context_to_machine(data_context)
    }
    pub(super) fn register_owned_view_model_rebind_dependents(&self) {
        if let Some(data_context) = self.owned_data_context.as_ref() {
            data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        }
    }
    pub(super) fn retain_owned_view_model_advance_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) {
        if data_context.is_empty() {
            self.active_owned_view_model_advance_context = None;
            return;
        }
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        for context in data_context.root_handles() {
            advance_context.extend(&context.borrow());
        }
        self.active_owned_view_model_advance_context = Some(advance_context);
    }
    fn sync_bindable_font_assets<F>(&mut self, mut resolve: F)
    where
        F: FnMut(&RuntimeBindableAssetDefaultViewModelSource) -> Option<RuntimeFontAssetValue>,
    {
        for bindable in &mut self.bindable_assets {
            let value = bindable
                .default_view_model_sources
                .iter()
                .filter(|source| {
                    data_bind_flags_apply_source_to_target(source.flags)
                        && source.value.font_value().is_some()
                })
                .filter_map(&mut resolve)
                .last();
            if let Some(value) = value {
                bindable.apply_font_value(&value);
            }
        }
    }
    fn sync_bindable_font_assets_from_default_context(&mut self) {
        self.sync_bindable_font_assets(|source| source.value.font_value().cloned());
    }
    fn sync_bindable_font_assets_from_imported_instance(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) {
        let instance_object = file
            .view_model(view_model_index)
            .and_then(|view_model| view_model.instances.into_iter().nth(instance_index))
            .map(|instance| instance.object);
        self.sync_bindable_font_assets(|source| {
            let source_object =
                file.data_context_view_model_property_for_instance(instance_object?, &source.path)?;
            (source_object.type_name == "ViewModelInstanceAssetFont")
                .then(|| source_object.uint_property("propertyValue"))
                .flatten()
                .map(RuntimeFontAssetValue::from_file_asset_index)
        });
    }
    pub(super) fn sync_bindable_font_assets_from_owned_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) {
        self.sync_bindable_font_assets(|source| {
            runtime_owned_font_asset_value_for_state_machine_source(context, &source.path)
        });
    }
    fn sync_bindable_font_assets_from_owned_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) {
        self.sync_bindable_font_assets(|source| {
            context_chain.iter().find_map(|context_path| {
                context.font_asset_value_by_context_source_path(
                    file,
                    context_path,
                    &source.path,
                    false,
                )
            })
        });
    }
    pub(super) fn sync_bindable_font_assets_from_owned_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) {
        self.sync_bindable_font_assets(|source| {
            data_context.resolved_property_path(&source.path).and_then(
                |(context, property_path)| {
                    context
                        .borrow()
                        .font_asset_value_by_property_path(&property_path)
                },
            )
        });
    }
    pub fn advance_data_context(&mut self) -> bool {
        self.collect_retained_owned_view_model_dirt();
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        // Pinned `StateMachineInstance::advancedDataContext` only consumes the
        // live ViewModel trigger values through `DataContext::advanced`.
        // DataBindContainer work remains owned by `updateDataBinds` during an
        // ordinary advance or the explicit public update API; doing it here
        // incorrectly applies queued target edits before the trigger reset
        // (`state_machine_instance.cpp:2587-2593`).
        self.reset_advanced_data_context();
        true
    }
    fn clear_view_model_listener_cell_bindings(&mut self) {
        for listener in &mut self.view_model_listeners {
            for binding in &mut listener.property_bindings {
                binding.cell_binding = None;
            }
        }
    }
}
