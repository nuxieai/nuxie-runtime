// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_artboard.cpp`.
// Artboard-valued cell identity, sentinel value, import, and clone behavior.

use crate::ArtboardInstance;

struct RuntimeBindableArtboardInner {
    name: String,
    owner: crate::artboard::RuntimeBindableArtboardOwner,
}

impl std::fmt::Debug for RuntimeBindableArtboardInner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeBindableArtboardInner")
            .field("name", &self.name)
            .field("owner", &self.owner)
            .finish()
    }
}

/// Retained safe-Rust analogue of one runtime `BindableArtboard`.
#[derive(Debug, Clone)]
pub struct RuntimeBindableArtboard {
    inner: Rc<RuntimeBindableArtboardInner>,
}

impl PartialEq for RuntimeBindableArtboard {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for RuntimeBindableArtboard {}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelArtboardBindingSource {
    state: Rc<RefCell<RuntimeOwnedViewModelArtboardState>>,
}

impl RuntimeOwnedViewModelArtboardBindingSource {
    fn new(state: Rc<RefCell<RuntimeOwnedViewModelArtboardState>>) -> Self {
        Self { state }
    }

    pub(crate) fn runtime_artboard(&self) -> Option<RuntimeBindableArtboard> {
        self.state.borrow().bindable_artboard.clone()
    }
}

impl RuntimeBindableArtboard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(RuntimeBindableArtboardInner {
                name: name.into(),
                owner: crate::artboard::RuntimeBindableArtboardOwner::new(None, None),
            }),
        }
    }

    #[doc(hidden)]
    pub fn new_with_artboard_instance(
        name: impl Into<String>,
        artboard: &ArtboardInstance,
    ) -> Self {
        Self {
            inner: Rc::new(RuntimeBindableArtboardInner {
                name: name.into(),
                owner: crate::artboard::RuntimeBindableArtboardOwner::new(
                    None,
                    Some(artboard.clone()),
                ),
            }),
        }
    }

    #[doc(hidden)]
    pub fn new_with_artboard_instance_and_file_authority(
        name: impl Into<String>,
        artboard: &ArtboardInstance,
        source_file_authority: Rc<dyn std::any::Any>,
    ) -> Self {
        Self {
            inner: Rc::new(RuntimeBindableArtboardInner {
                name: name.into(),
                owner: crate::artboard::RuntimeBindableArtboardOwner::new(
                    Some(source_file_authority),
                    Some(artboard.clone()),
                ),
            }),
        }
    }

    /// Refresh the retained source occurrence before publishing this stable
    /// bindable identity through a host command.
    #[doc(hidden)]
    pub fn refresh_artboard_instance(&self, artboard: &ArtboardInstance) {
        self.inner.owner.replace_artboard(artboard.clone());
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    #[doc(hidden)]
    pub fn artboard_instance(&self) -> Option<ArtboardInstance> {
        self.inner.owner.artboard()
    }

    /// Whether the retained pointer-style source currently names a concrete
    /// Artboard occurrence. This is the allocation-free counterpart of pinned
    /// `ScriptInputArtboard::validateHydrationPrerequisites`; callers must take
    /// the actual fresh clone later at the authored phase-two position.
    #[doc(hidden)]
    pub fn has_artboard_instance(&self) -> bool {
        self.inner.owner.has_artboard()
    }

    #[doc(hidden)]
    pub fn source_file_authority<T: 'static>(&self) -> Option<Rc<T>> {
        self.inner.owner.source_file::<T>()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeOwnedViewModelArtboardState {
    pub(crate) bindable_artboard: Option<RuntimeBindableArtboard>,
    pub(crate) bound_view_model_instance: Option<RuntimeOwnedViewModelHandle>,
}

/// Pinned `ViewModelInstanceArtboard::asset`, shared by the owned graph and
/// runtime facade so both live consumers execute the nominal owner's exact
/// write-before-dirt sequence.
pub(super) fn view_model_instance_artboard_asset(
    cell: &RuntimeViewModelCell,
    state: &Rc<RefCell<RuntimeOwnedViewModelArtboardState>>,
    value: Option<RuntimeBindableArtboard>,
) {
    if !matches!(cell.value(), RuntimeViewModelCellValue::Artboard(u32::MAX)) {
        state.borrow_mut().bindable_artboard = None;
    }
    cell.set_value(RuntimeViewModelCellValue::Artboard(u32::MAX));
    state.borrow_mut().bindable_artboard = value;
    cell.notify_bindings_value_changed();
}

#[derive(Debug)]
struct RuntimeOwnedViewModelArtboard {
    property_index: usize,
    cell: RuntimeViewModelCell,
    runtime_state: Rc<RefCell<RuntimeOwnedViewModelArtboardState>>,
}

impl RuntimeOwnedViewModelArtboard {
    fn new(property_index: usize, value: u64) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Artboard(
                owned_scalar_u32_payload(value),
            )),
            runtime_state: Rc::new(RefCell::new(RuntimeOwnedViewModelArtboardState::default())),
        }
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Artboard(value) => u64::from(value),
            _ => unreachable!("owned artboard slot holds a non-artboard cell"),
        }
    }

    fn set_value(&mut self, value: u64) -> bool {
        if self.value() == value {
            return false;
        }
        self.runtime_state.borrow_mut().bindable_artboard = None;
        self.cell.set_value(RuntimeViewModelCellValue::Artboard(
            owned_scalar_u32_payload(value),
        ));
        true
    }

    /// Pinned `ViewModelInstanceArtboard::asset`. The generated
    /// `propertyValue(-1)` setter runs first and therefore clears the old
    /// bindable before its first dirt cascade when the serialized value was
    /// not already the sentinel. The new bindable is then installed and the
    /// explicit second `Bindings` cascade always runs, including for the same
    /// pointer and for `nullptr`.
    fn set_asset(&mut self, value: Option<RuntimeBindableArtboard>) {
        view_model_instance_artboard_asset(&self.cell, &self.runtime_state, value);
    }

    /// Pinned `ViewModelInstanceArtboard::boundViewModelInstance` retains the
    /// supplied instance without publishing value dirt.
    fn set_bound_view_model_instance(&self, value: Option<RuntimeOwnedViewModelHandle>) {
        self.runtime_state.borrow_mut().bound_view_model_instance = value;
    }

    /// Pinned `ViewModelInstanceArtboard::advanced`: advance the retained
    /// bound instance before acknowledging this artboard value itself.
    fn advanced_data_context(&self) {
        let bound = self
            .runtime_state
            .borrow()
            .bound_view_model_instance
            .clone();
        if let Some(bound) = bound {
            bound.borrow_mut().advanced_data_context();
        }
        self.cell.advanced();
    }

    /// Script-frame counterpart of `advanced_data_context`. The bound
    /// instance advances at the authored position before this value is
    /// acknowledged.
    fn advance_script_frame(&self, visited: &mut BTreeSet<u64>) -> bool {
        let mut changed = false;
        let bound = self
            .runtime_state
            .borrow()
            .bound_view_model_instance
            .clone();
        if let Some(bound) = bound {
            changed |=
                RuntimeOwnedViewModelInstance::advance_script_frame(&bound.shared(), visited);
        }
        self.cell.advanced();
        changed
    }

    fn runtime_state(&self) -> Rc<RefCell<RuntimeOwnedViewModelArtboardState>> {
        Rc::clone(&self.runtime_state)
    }
}

impl Clone for RuntimeOwnedViewModelArtboard {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyArtboard").then_some(
                        // C++ `ViewModelInstanceArtboardBase` initializes
                        // an unassigned property to its `-1` sentinel.
                        RuntimeOwnedViewModelArtboard::new(property_index, u64::from(u32::MAX)),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_artboards_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_artboard_index_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelArtboard::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_artboards_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
