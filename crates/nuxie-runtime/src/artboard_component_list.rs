// Mounted `ArtboardComponentList` row ownership.
//
// Direct port owner for pinned C++ `src/artboard_component_list.cpp`.

#[derive(Debug, Clone)]
pub(crate) struct RuntimeComponentListItemInstance {
    // C++ erases row state machines before row ArtboardInstances so listener
    // groups cannot observe destroyed FocusData.
    pub(crate) state_machines: Vec<StateMachineInstance>,
    pub(crate) child: Box<ArtboardInstance>,
    pub(crate) render_resources: RefCell<crate::draw::RuntimeOccurrenceRenderResources>,
    pub(crate) context: RuntimeOwnedViewModelHandle,
    pub(crate) context_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink,
    pub(crate) draw_index_sink: Option<crate::view_model_cell::RuntimeCellDirtSink>,
    pub(crate) occurrence_identity: u64,
    pub(crate) logical_index: usize,
    pub(crate) settled_layout_size: Cell<Option<(f32, f32)>>,
    pub(crate) transform: Mat2D,
    pub(crate) render_cache_revision: u64,
}

impl RuntimeComponentListItemInstance {
    fn context_is_current(&self, context: &RuntimeOwnedViewModelHandle) -> bool {
        self.context.ptr_eq(context)
            && !self
                .context_rebind_sink
                .peek_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
    }

    fn consume_context_rebind_dirt(&self) {
        self.context_rebind_sink.take_dirt();
    }

    /// Apply the C++ property-recorder reset to a pooled occurrence.
    ///
    /// Rust can restore the authored clone in one ownership-safe move. Keep
    /// the pooled Box allocation, Artboard occurrence identity, and retained
    /// renderer resources; C++ rewinds the occurrence's authored properties
    /// without replacing its RenderPath/RenderPaint owners.
    fn restore_from_fresh(&mut self, mut fresh: Self) {
        fresh
            .child
            .runtime_shapes
            .adopt_pooled_backend_owners(&mut self.child.runtime_shapes);
        fresh
            .child
            .runtime_drawables
            .adopt_pooled_backend_owners(&mut self.child.runtime_drawables);
        std::mem::swap(
            &mut self.child.instance_identity,
            &mut fresh.child.instance_identity,
        );
        *self.child = *fresh.child;
        self.state_machines.clear();
        self.state_machines.append(&mut fresh.state_machines);
        self.context = fresh.context;
        self.context_rebind_sink = fresh.context_rebind_sink;
        self.draw_index_sink = fresh.draw_index_sink;
        self.occurrence_identity = fresh.occurrence_identity;
        self.logical_index = fresh.logical_index;
        self.settled_layout_size = fresh.settled_layout_size;
        self.transform = fresh.transform;
        self.render_cache_revision = fresh.render_cache_revision;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeComponentListResourcePools {
    items: BTreeMap<(usize, u32), Vec<RuntimeComponentListItemInstance>>,
}

impl RuntimeComponentListResourcePools {
    fn take(
        &mut self,
        list_local_id: usize,
        source_global_id: u32,
    ) -> Option<RuntimeComponentListItemInstance> {
        self.items
            .get_mut(&(list_local_id, source_global_id))
            .and_then(Vec::pop)
    }

    fn put(&mut self, list_local_id: usize, item: RuntimeComponentListItemInstance) {
        self.items
            .entry((list_local_id, item.child.graph_global_id))
            .or_default()
            .push(item);
    }

    #[cfg(test)]
    fn count(&self, list_local_id: usize, source_global_id: u32) -> usize {
        self.items
            .get(&(list_local_id, source_global_id))
            .map_or(0, Vec::len)
    }
}

fn component_list_draw_index_sink(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelHandle,
) -> Option<crate::view_model_cell::RuntimeCellDirtSink> {
    let property_name = file
        .view_model_property_for_symbol(context.borrow().view_model_index(), 16)?
        .string_property("name")?;
    let cell = context
        .borrow()
        .number_cell_by_property_name(property_name)?;
    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
    cell.add_dependent(&sink);
    Some(sink)
}

/// Logical list topology retained independently from the mounted window,
/// matching C++ `m_listItems` and `m_artboardSizes`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeComponentListLogicalItem {
    pub(crate) occurrence_identity: u64,
    pub(crate) context: RuntimeOwnedViewModelHandle,
    pub(crate) size: (f32, f32),
    pub(crate) mapped_artboard_global: Option<u32>,
}

#[cfg(test)]
fn component_list_contexts_retain_same_handles(
    existing: &[RuntimeComponentListItemInstance],
    incoming: &[RuntimeOwnedViewModelHandle],
) -> bool {
    existing.len() == incoming.len()
        && existing
            .iter()
            .zip(incoming)
            .all(|(item, context)| item.context.ptr_eq(context))
}

fn component_list_default_state_machine_index(
    default_state_machine_id: Option<u64>,
    state_machine_count: usize,
) -> usize {
    default_state_machine_id
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < state_machine_count)
        .unwrap_or(0)
}
