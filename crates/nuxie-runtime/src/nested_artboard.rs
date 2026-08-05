// Mounted `NestedArtboard` occurrence ownership.
//
// This is the direct Rust owner for pinned C++ `src/nested_artboard.cpp`.
// The parent `ArtboardInstance` includes this file so the occurrence can keep
// borrowing the surrounding private runtime vocabulary without recreating a
// parallel public abstraction.

#[derive(Debug)]
pub(crate) struct RuntimeNestedArtboardInstance {
    // Rust drops fields in declaration order. C++ releases nested animations
    // (including StateMachineInstances that can reference m_Instance) before
    // destroying m_Instance (`nested_artboard.cpp:48-64`).
    pub(crate) animations: Vec<RuntimeNestedAnimationInstance>,
    pub(crate) child: Box<ArtboardInstance>,
    pub(crate) render_cache_revision: u64,
    /// C++ child objects own their backend members. This sidecar follows the
    /// mounted occurrence through replacement/drop and is rebuilt on clone.
    pub(crate) render_resources: RefCell<crate::draw::RuntimeOccurrenceRenderResources>,
    /// Initial paint state retained until `NestedArtboardLayout` transfers its
    /// layout data to this exact mounted child.
    pub(crate) initial_layout_paint_frame: RefCell<Option<RuntimeInitialNestedLayoutPaintFrame>>,
    /// Intrinsic Hug size observed before this child's root layout node was
    /// transferred to its parent-owned layout tree.
    pub(crate) transferred_hug_size: Cell<(Option<f32>, Option<f32>)>,
    /// Last child layout generation represented by `transferred_hug_size`.
    pub(crate) transferred_hug_layout_generation: Cell<u64>,
    pub(crate) layout_data_transferred: bool,
    layout_data_transfer_key: Option<RuntimeNestedLayoutDataTransferKey>,
    pub(crate) data_bind_path_ids: Option<Vec<u32>>,
    pub(crate) data_bind_path_is_relative: bool,
    pub(crate) stateful_view_model_instance_local: Option<usize>,
    pub(crate) stateful_view_model_instance_locals_by_id: BTreeMap<u32, usize>,
    pub(crate) stateful_view_model_context: Option<RuntimeOwnedViewModelHandle>,
    pub(crate) stateful_global_view_model_contexts: BTreeMap<usize, RuntimeOwnedViewModelHandle>,
    pub(crate) data_bind_property_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_image_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_context_source_locals_by_path: BTreeMap<Vec<u32>, usize>,
    is_paused: bool,
    speed: f32,
    quantize: f32,
    cumulated_seconds: f32,
}

impl Clone for RuntimeNestedArtboardInstance {
    fn clone(&self) -> Self {
        // A normal Artboard clone creates a new mounted occurrence. Pinned C++
        // gives it a fresh `takeLayoutData()` lifecycle and cold-clones nested
        // state machines against the new child owner.
        let mut child = self.child.as_ref().clone();
        child.reset_layout_constraint_bounds_for_new_occurrence();
        let animations = self
            .animations
            .iter()
            .map(|animation| match animation {
                RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence.cold_clone(&mut child))
                }
                animation => animation.clone(),
            })
            .collect();
        Self {
            animations,
            child: Box::new(child),
            render_cache_revision: self.render_cache_revision,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            transferred_hug_size: Cell::new((None, None)),
            transferred_hug_layout_generation: Cell::new(0),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: self.data_bind_path_ids.clone(),
            data_bind_path_is_relative: self.data_bind_path_is_relative,
            stateful_view_model_instance_local: self.stateful_view_model_instance_local,
            stateful_view_model_instance_locals_by_id: self
                .stateful_view_model_instance_locals_by_id
                .clone(),
            stateful_view_model_context: self
                .stateful_view_model_context
                .as_ref()
                .map(|context| RuntimeOwnedViewModelHandle::new(context.borrow().clone())),
            stateful_global_view_model_contexts: self
                .stateful_global_view_model_contexts
                .iter()
                .map(|(&view_model_index, context)| {
                    (
                        view_model_index,
                        RuntimeOwnedViewModelHandle::new(context.borrow().clone()),
                    )
                })
                .collect(),
            data_bind_property_source_locals: self.data_bind_property_source_locals.clone(),
            data_bind_image_source_locals: self.data_bind_image_source_locals.clone(),
            data_bind_context_source_locals_by_path: self
                .data_bind_context_source_locals_by_path
                .clone(),
            is_paused: self.is_paused,
            speed: self.speed,
            quantize: self.quantize,
            cumulated_seconds: self.cumulated_seconds,
        }
    }
}

/// Mounted occurrences retained contiguously like C++
/// `Artboard::m_NestedArtboards`, with a sparse local-id side index.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeNestedArtboards {
    entries: Vec<(usize, RuntimeNestedArtboardInstance)>,
    entry_by_local: Vec<Option<usize>>,
    state_machine_owner_by_local: Vec<Option<(usize, usize)>>,
}

impl RuntimeNestedArtboards {
    pub(crate) fn get(&self, local_id: &usize) -> Option<&RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get(*local_id).copied().flatten()?;
        self.entries.get(entry).map(|(_, nested)| nested)
    }

    pub(crate) fn get_mut(
        &mut self,
        local_id: &usize,
    ) -> Option<&mut RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get(*local_id).copied().flatten()?;
        self.entries.get_mut(entry).map(|(_, nested)| nested)
    }

    fn contains_key(&self, local_id: &usize) -> bool {
        self.entry_by_local
            .get(*local_id)
            .is_some_and(Option::is_some)
    }

    fn insert(
        &mut self,
        local_id: usize,
        nested: RuntimeNestedArtboardInstance,
    ) -> Option<RuntimeNestedArtboardInstance> {
        if self.entry_by_local.len() <= local_id {
            self.entry_by_local.resize(local_id.saturating_add(1), None);
        }
        if let Some(entry) = self.entry_by_local[local_id] {
            let previous = std::mem::replace(&mut self.entries[entry].1, nested);
            self.rebuild_state_machine_index();
            return Some(previous);
        }

        let entry = self
            .entries
            .binary_search_by_key(&local_id, |(candidate, _)| *candidate)
            .unwrap_or_else(|entry| entry);
        self.entries.insert(entry, (local_id, nested));
        for (entry, (local_id, _)) in self.entries.iter().enumerate().skip(entry) {
            self.entry_by_local[*local_id] = Some(entry);
        }
        self.rebuild_state_machine_index();
        None
    }

    fn remove(&mut self, local_id: &usize) -> Option<RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get_mut(*local_id)?.take()?;
        let (_, nested) = self.entries.remove(entry);
        for (entry, (local_id, _)) in self.entries.iter().enumerate().skip(entry) {
            self.entry_by_local[*local_id] = Some(entry);
        }
        self.rebuild_state_machine_index();
        Some(nested)
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &usize> {
        self.entries.iter().map(|(local_id, _)| local_id)
    }

    fn iter(&self) -> impl Iterator<Item = (&usize, &RuntimeNestedArtboardInstance)> {
        self.entries
            .iter()
            .map(|(local_id, nested)| (local_id, nested))
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &RuntimeNestedArtboardInstance> {
        self.entries.iter().map(|(_, nested)| nested)
    }

    pub(crate) fn values_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RuntimeNestedArtboardInstance> {
        self.entries.iter_mut().map(|(_, nested)| nested)
    }

    pub(crate) fn state_machine(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeNestedStateMachineInstance> {
        let (host_entry, animation_index) = self
            .state_machine_owner_by_local
            .get(local_id)
            .copied()
            .flatten()?;
        match self
            .entries
            .get(host_entry)?
            .1
            .animations
            .get(animation_index)?
        {
            RuntimeNestedAnimationInstance::StateMachine(occurrence)
                if occurrence.local_id() == local_id =>
            {
                Some(occurrence)
            }
            _ => None,
        }
    }

    pub(crate) fn state_machine_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut RuntimeNestedStateMachineInstance> {
        let (host_entry, animation_index) = self
            .state_machine_owner_by_local
            .get(local_id)
            .copied()
            .flatten()?;
        match self
            .entries
            .get_mut(host_entry)?
            .1
            .animations
            .get_mut(animation_index)?
        {
            RuntimeNestedAnimationInstance::StateMachine(occurrence)
                if occurrence.local_id() == local_id =>
            {
                Some(occurrence)
            }
            _ => None,
        }
    }

    fn rebuild_state_machine_index(&mut self) {
        self.state_machine_owner_by_local.clear();
        for (host_entry, (_, nested)) in self.entries.iter().enumerate() {
            for (animation_index, animation) in nested.animations.iter().enumerate() {
                let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                    continue;
                };
                let local_id = occurrence.local_id();
                if self.state_machine_owner_by_local.len() <= local_id {
                    self.state_machine_owner_by_local
                        .resize(local_id.saturating_add(1), None);
                }
                self.state_machine_owner_by_local[local_id] = Some((host_entry, animation_index));
            }
        }
    }
}

impl std::ops::Index<&usize> for RuntimeNestedArtboards {
    type Output = RuntimeNestedArtboardInstance;

    fn index(&self, local_id: &usize) -> &Self::Output {
        self.get(local_id)
            .unwrap_or_else(|| panic!("no nested artboard mounted at local id {local_id}"))
    }
}
