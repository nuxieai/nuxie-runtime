// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel.cpp`.
// Ordered authored/default/global ViewModel catalogs and selected instances.

/// The ordered set of owned view-model instances visible to an artboard or
/// state machine.
///
/// Rive resolves the main instance first, followed by global slots in file
/// view-model order. A global slot is addressed by the declared global view
/// model, independently of the view model that produced the occupying
/// instance. That distinction is what permits a different view model to be
/// installed as an override for a global slot.
#[derive(Debug, Clone, Default)]
pub struct RuntimeOwnedViewModelContext {
    main: Option<RuntimeOwnedViewModelHandle>,
    global_slots: BTreeMap<usize, RuntimeOwnedViewModelHandle>,
}

impl RuntimeOwnedViewModelContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_main(main: RuntimeOwnedViewModelInstance) -> Self {
        Self::from_main_handle(RuntimeOwnedViewModelHandle::new(main))
    }

    pub fn from_main_handle(main: RuntimeOwnedViewModelHandle) -> Self {
        Self {
            main: Some(main),
            global_slots: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.main.is_none() && self.global_slots.is_empty()
    }

    pub fn main(&self) -> Option<Ref<'_, RuntimeOwnedViewModelInstance>> {
        self.main.as_ref().map(RuntimeOwnedViewModelHandle::borrow)
    }

    pub fn main_mut(&self) -> Option<RefMut<'_, RuntimeOwnedViewModelInstance>> {
        self.main
            .as_ref()
            .map(RuntimeOwnedViewModelHandle::borrow_mut)
    }

    pub fn main_handle(&self) -> Option<&RuntimeOwnedViewModelHandle> {
        self.main.as_ref()
    }

    pub fn set_main(&mut self, main: RuntimeOwnedViewModelInstance) {
        self.set_main_handle(RuntimeOwnedViewModelHandle::new(main));
    }

    pub fn set_main_handle(&mut self, main: RuntimeOwnedViewModelHandle) {
        self.main = Some(main);
    }

    pub fn take_main(&mut self) -> Option<RuntimeOwnedViewModelHandle> {
        self.main.take()
    }

    /// Returns instances in C++ `DataContext::viewModelInstances()` order:
    /// main first, then globals by their file view-model slot.
    pub fn instances(&self) -> impl Iterator<Item = Ref<'_, RuntimeOwnedViewModelInstance>> {
        self.handles().map(RuntimeOwnedViewModelHandle::borrow)
    }

    pub fn handles(&self) -> impl Iterator<Item = &RuntimeOwnedViewModelHandle> {
        self.main.iter().chain(self.global_slots.values())
    }

    pub fn global_slot(
        &self,
        view_model_index: usize,
    ) -> Option<Ref<'_, RuntimeOwnedViewModelInstance>> {
        self.global_slots
            .get(&view_model_index)
            .map(RuntimeOwnedViewModelHandle::borrow)
    }

    pub fn global_slot_mut(
        &self,
        view_model_index: usize,
    ) -> Option<RefMut<'_, RuntimeOwnedViewModelInstance>> {
        self.global_slots
            .get(&view_model_index)
            .map(RuntimeOwnedViewModelHandle::borrow_mut)
    }

    pub fn global_slot_handle(
        &self,
        view_model_index: usize,
    ) -> Option<&RuntimeOwnedViewModelHandle> {
        self.global_slots.get(&view_model_index)
    }

    pub(crate) fn global_slot_handles(
        &self,
    ) -> impl Iterator<Item = (usize, &RuntimeOwnedViewModelHandle)> {
        self.global_slots
            .iter()
            .map(|(&slot, handle)| (slot, handle))
    }

    pub fn global_named(
        &self,
        file: &RuntimeFile,
        name: &str,
    ) -> Option<Ref<'_, RuntimeOwnedViewModelInstance>> {
        let slot = runtime_global_view_model_index_named(file, name)?;
        self.global_slot(slot)
    }

    pub fn global_named_mut(
        &self,
        file: &RuntimeFile,
        name: &str,
    ) -> Option<RefMut<'_, RuntimeOwnedViewModelInstance>> {
        let slot = runtime_global_view_model_index_named(file, name)?;
        self.global_slot_mut(slot)
    }

    /// Installs `instance` into the named global slot. The instance's own view
    /// model intentionally need not match the slot's declared view model.
    pub fn set_global_named(
        &mut self,
        file: &RuntimeFile,
        name: &str,
        instance: RuntimeOwnedViewModelInstance,
    ) -> bool {
        let Some(slot) = runtime_global_view_model_index_named(file, name) else {
            return false;
        };
        self.global_slots
            .insert(slot, RuntimeOwnedViewModelHandle::new(instance));
        true
    }

    pub fn set_global_named_handle(
        &mut self,
        file: &RuntimeFile,
        name: &str,
        instance: RuntimeOwnedViewModelHandle,
    ) -> bool {
        let Some(slot) = runtime_global_view_model_index_named(file, name) else {
            return false;
        };
        self.global_slots.insert(slot, instance);
        true
    }

    /// Empty one named global slot while preserving every other occupant.
    /// A valid already-empty slot still succeeds.
    pub fn unset_global_named(&mut self, file: &RuntimeFile, name: &str) -> bool {
        let Some(slot) = runtime_global_view_model_index_named(file, name) else {
            return false;
        };
        self.global_slots.remove(&slot);
        true
    }

    pub fn set_global_slot(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance: RuntimeOwnedViewModelInstance,
    ) -> bool {
        if !runtime_view_model_is_global(file, view_model_index) {
            return false;
        }
        self.global_slots
            .insert(view_model_index, RuntimeOwnedViewModelHandle::new(instance));
        true
    }

    pub fn set_global_slot_handle(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance: RuntimeOwnedViewModelHandle,
    ) -> bool {
        if !runtime_view_model_is_global(file, view_model_index) {
            return false;
        }
        self.global_slots.insert(view_model_index, instance);
        true
    }

    /// Completes any missing instances the same way C++ state-machine `bind()`
    /// does: the artboard's main default first, then every global default.
    /// Existing slots, including cross-view-model overrides, are preserved.
    pub fn complete_for_artboard(&mut self, file: &RuntimeFile, artboard_index: usize) -> bool {
        let main_view_model_index = file
            .resolved_view_model_for_artboard(artboard_index)
            .map(|view_model| view_model.view_model_index);
        self.complete(file, main_view_model_index)
    }

    pub fn complete(&mut self, file: &RuntimeFile, main_view_model_index: Option<usize>) -> bool {
        let mut changed = false;
        if self.main.is_none() {
            if let Some(view_model_index) = main_view_model_index {
                if let Some(instance) =
                    runtime_default_owned_view_model_instance(file, view_model_index)
                {
                    self.main = Some(RuntimeOwnedViewModelHandle::new(instance));
                    changed = true;
                }
            }
        }
        for view_model_index in runtime_global_view_model_indices(file) {
            if self.global_slots.contains_key(&view_model_index) {
                continue;
            }
            let Some(instance) = runtime_default_owned_view_model_instance(file, view_model_index)
            else {
                continue;
            };
            self.global_slots
                .insert(view_model_index, RuntimeOwnedViewModelHandle::new(instance));
            changed = true;
        }
        changed
    }

    /// C++ `DataContext::getViewModelProperty(path)` over this context's
    /// retained instances (#RB-1 e3): try each instance in canonical order —
    /// main first, then globals by slot — and return the retained CELL, not
    /// a copy. This public composite is one local instance list; the production
    /// owned DataContext carrier supplies its optional parent link.
    pub fn cell_for_source_path(&self, path: &[u32]) -> Option<RuntimeViewModelCell> {
        self.handles()
            .find_map(|handle| handle.borrow().cell_for_source_path(path))
    }
}

fn runtime_default_owned_view_model_instance(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Option<RuntimeOwnedViewModelInstance> {
    RuntimeOwnedViewModelInstance::from_instance(file, view_model_index, 0)
        .or_else(|| RuntimeOwnedViewModelInstance::new(file, view_model_index))
}

fn runtime_view_model_is_global(file: &RuntimeFile, view_model_index: usize) -> bool {
    file.view_model(view_model_index)
        .and_then(|view_model| view_model.object.uint_property("viewModelType"))
        == Some(2)
}

pub fn runtime_global_view_model_indices(file: &RuntimeFile) -> Vec<usize> {
    file.view_models()
        .iter()
        .enumerate()
        .filter_map(|(index, view_model)| {
            (view_model.object.uint_property("viewModelType") == Some(2)).then_some(index)
        })
        .collect()
}

pub fn runtime_global_view_model_names(file: &RuntimeFile) -> Vec<String> {
    runtime_global_view_model_indices(file)
        .into_iter()
        .filter_map(|index| {
            file.view_model(index)
                .and_then(|view_model| view_model.object.string_property("name"))
                .map(str::to_owned)
        })
        .collect()
}

fn runtime_global_view_model_index_named(file: &RuntimeFile, name: &str) -> Option<usize> {
    file.view_models()
        .iter()
        .enumerate()
        .find_map(|(index, view_model)| {
            (view_model.object.uint_property("viewModelType") == Some(2)
                && view_model.object.string_property("name") == Some(name))
            .then_some(index)
        })
}

/// A schema-aware view into one retained owned view-model graph.
///
/// The root handle preserves graph identity while `scope_path` identifies the
/// actively selected nested view model. This avoids materializing a detached
/// child instance merely to expose a nested scripting or binding context.
#[derive(Debug, Clone)]
pub struct RuntimeOwnedViewModelContextHandle {
    file: Option<Rc<RuntimeFile>>,
    root: RuntimeOwnedViewModelHandle,
    scope_path: Vec<usize>,
}

impl RuntimeOwnedViewModelContextHandle {
    pub fn root(file: &RuntimeFile, root: RuntimeOwnedViewModelHandle) -> Self {
        Self {
            file: Some(Rc::new(file.clone())),
            root,
            scope_path: Vec::new(),
        }
    }

    pub(crate) fn root_without_file(root: RuntimeOwnedViewModelHandle) -> Self {
        Self {
            file: None,
            root,
            scope_path: Vec::new(),
        }
    }

    pub(crate) fn scoped(&self, scope_path: Vec<usize>) -> Option<Self> {
        self.root
            .borrow()
            .view_model_index_by_property_path(&scope_path)?;
        Some(Self {
            file: self.file.as_ref().map(Rc::clone),
            root: self.root.clone(),
            scope_path,
        })
    }

    pub fn root_handle(&self) -> RuntimeOwnedViewModelHandle {
        self.root.clone()
    }

    pub fn scope_path(&self) -> &[usize] {
        &self.scope_path
    }

    pub fn file(&self) -> Option<&RuntimeFile> {
        self.file.as_deref()
    }

    pub fn view_model_index(&self) -> Option<usize> {
        self.root
            .borrow()
            .view_model_index_by_property_path(&self.scope_path)
    }

    pub fn is_root(&self) -> bool {
        self.scope_path.is_empty()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.root.ptr_eq(&other.root) && self.scope_path == other.scope_path
    }

    pub fn shares_root_with(&self, root: &RuntimeOwnedViewModelHandle) -> bool {
        self.root.ptr_eq(root)
    }

    pub fn detached_snapshot(&self) -> Option<RuntimeOwnedViewModelInstance> {
        if self.scope_path.is_empty() {
            return Some(self.root.borrow().clone());
        }
        self.root
            .borrow()
            .nested_instance_by_property_path(&self.scope_path)
    }
}
