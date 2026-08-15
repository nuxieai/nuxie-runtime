// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_artboard.cpp`.
// Artboard-valued cell identity, sentinel value, import, and clone behavior.

use crate::ArtboardInstance;

#[derive(Debug)]
struct RuntimeBindableArtboardInner {
    name: String,
    // C++ `BindableArtboard` retains the concrete source ArtboardInstance, not
    // a file-local index. Keeping the cold-clone source here preserves both
    // cross-file identity and live generated properties such as resized bounds.
    source: RefCell<Option<ArtboardInstance>>,
}

/// Retained safe-Rust analogue of one runtime `BindableArtboard`.
#[derive(Debug, Clone)]
pub struct RuntimeBindableArtboard {
    inner: Rc<RuntimeBindableArtboardInner>,
}

impl RuntimeBindableArtboard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(RuntimeBindableArtboardInner {
                name: name.into(),
                source: RefCell::new(None),
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
                source: RefCell::new(Some(artboard.clone())),
            }),
        }
    }

    /// Refresh the retained source occurrence before publishing this stable
    /// bindable identity through a host command.
    #[doc(hidden)]
    pub fn refresh_artboard_instance(&self, artboard: &ArtboardInstance) {
        self.inner.source.replace(Some(artboard.clone()));
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    pub(crate) fn artboard_instance(&self) -> Option<ArtboardInstance> {
        self.inner.source.borrow().clone()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeOwnedViewModelArtboardState {
    pub(crate) bindable_artboard: Option<RuntimeBindableArtboard>,
    pub(crate) bound_view_model_instance: Option<RuntimeOwnedViewModelHandle>,
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

    fn runtime_state(&self) -> Rc<RefCell<RuntimeOwnedViewModelArtboardState>> {
        Rc::clone(&self.runtime_state)
    }

    fn notify_bindings_value_changed(&self) {
        self.cell.notify_bindings_value_changed();
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
