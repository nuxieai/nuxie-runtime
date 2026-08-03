// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_value.cpp`.
// Common retained cell/dependency identity and structural source dispatch.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RuntimeOwnedViewModelValueKind {
    Number,
    Boolean,
    String,
    Color,
    Enum,
    SymbolListIndex,
    List,
    Asset,
    FontAsset,
    BlobAsset,
    Artboard,
    Trigger,
    ViewModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeOwnedViewModelValueOccurrence {
    kind: RuntimeOwnedViewModelValueKind,
    slot_index: usize,
    property_index: usize,
}

fn runtime_owned_view_model_value_kind(type_name: &str) -> Option<RuntimeOwnedViewModelValueKind> {
    Some(match type_name {
        "ViewModelPropertyNumber" | "ViewModelInstanceNumber" => {
            RuntimeOwnedViewModelValueKind::Number
        }
        "ViewModelPropertyBoolean" | "ViewModelInstanceBoolean" => {
            RuntimeOwnedViewModelValueKind::Boolean
        }
        "ViewModelPropertyString" | "ViewModelInstanceString" => {
            RuntimeOwnedViewModelValueKind::String
        }
        "ViewModelPropertyColor" | "ViewModelInstanceColor" => {
            RuntimeOwnedViewModelValueKind::Color
        }
        "ViewModelPropertyEnum"
        | "ViewModelPropertyEnumCustom"
        | "ViewModelPropertyEnumSystem"
        | "ViewModelInstanceEnum" => {
            RuntimeOwnedViewModelValueKind::Enum
        }
        "ViewModelPropertySymbolListIndex" | "ViewModelInstanceSymbolListIndex" => {
            RuntimeOwnedViewModelValueKind::SymbolListIndex
        }
        "ViewModelPropertyList" | "ViewModelInstanceList" => RuntimeOwnedViewModelValueKind::List,
        "ViewModelPropertyAsset"
        | "ViewModelPropertyAssetImage"
        | "ViewModelInstanceAsset"
        | "ViewModelInstanceAssetImage" => RuntimeOwnedViewModelValueKind::Asset,
        "ViewModelPropertyAssetFont" | "ViewModelInstanceAssetFont" => {
            RuntimeOwnedViewModelValueKind::FontAsset
        }
        "ViewModelPropertyAssetBlob" | "ViewModelInstanceAssetBlob" => {
            RuntimeOwnedViewModelValueKind::BlobAsset
        }
        "ViewModelPropertyArtboard" | "ViewModelInstanceArtboard" => {
            RuntimeOwnedViewModelValueKind::Artboard
        }
        "ViewModelPropertyTrigger" | "ViewModelInstanceTrigger" => {
            RuntimeOwnedViewModelValueKind::Trigger
        }
        "ViewModelPropertyViewModel" | "ViewModelInstanceViewModel" => {
            RuntimeOwnedViewModelValueKind::ViewModel
        }
        _ => return None,
    })
}

fn runtime_owned_view_model_instance_value_objects<'a>(
    file: &'a RuntimeFile,
    view_model_index: usize,
    instance: &RuntimeObject,
) -> Vec<&'a RuntimeObject> {
    file.view_model(view_model_index)
        .and_then(|view_model| {
            view_model
                .instances
                .into_iter()
                .find(|candidate| candidate.object.id == instance.id)
        })
        .map(|instance| {
            instance
                .values
                .into_iter()
                .map(|value| value.object)
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_value_order(
    file: &RuntimeFile,
    view_model_index: usize,
    instance: Option<&RuntimeObject>,
) -> Vec<RuntimeOwnedViewModelValueOccurrence> {
    let ordered = if let Some(instance) = instance {
        runtime_owned_view_model_instance_value_objects(file, view_model_index, instance)
    } else {
        file.view_model(view_model_index)
            .map(|view_model| view_model.properties)
            .unwrap_or_default()
    };
    let mut counts = BTreeMap::<RuntimeOwnedViewModelValueKind, usize>::new();
    ordered
        .into_iter()
        .enumerate()
        .filter_map(|(schema_index, value)| {
            let kind = runtime_owned_view_model_value_kind(value.type_name)?;
            let property_index = if instance.is_some() {
                usize::try_from(value.uint_property("viewModelPropertyId")?).ok()?
            } else {
                schema_index
            };
            let slot_index = counts.entry(kind).or_default();
            let occurrence = RuntimeOwnedViewModelValueOccurrence {
                kind,
                slot_index: *slot_index,
                property_index,
            };
            *slot_index += 1;
            Some(occurrence)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeOwnedViewModelPropertyKind {
    String,
    Boolean,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelEndpointState {
    value: RuntimeViewModelPointer,
    linked_instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    cell: RuntimeViewModelCell,
}

/// Retained mutable state for one ViewModel-valued property.
///
/// C++ `ViewModelInstanceViewModel` keeps one retained child endpoint. Path
/// traversal follows `linked_instance` directly, so child replacement is
/// visible immediately even while the owning root is already borrowed.
/// Ordinary `Clone` detaches this endpoint before graph links are remapped.
#[derive(Debug)]
struct RuntimeOwnedViewModelEndpoint {
    state: Rc<RefCell<RuntimeOwnedViewModelEndpointState>>,
}

impl RuntimeOwnedViewModelEndpoint {
    fn new(value: RuntimeViewModelPointer) -> Self {
        Self {
            state: Rc::new(RefCell::new(RuntimeOwnedViewModelEndpointState {
                value,
                linked_instance: None,
                cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::ViewModel),
            })),
        }
    }

    fn value(&self) -> RuntimeViewModelPointer {
        self.state.borrow().value
    }

    fn cell(&self) -> RuntimeViewModelCell {
        self.state.borrow().cell.clone()
    }

    fn retained_source(&self) -> RuntimeOwnedViewModelStructuralSource {
        RuntimeOwnedViewModelStructuralSource::ViewModel(RuntimeOwnedViewModelEndpointSource {
            state: Rc::clone(&self.state),
        })
    }

    fn select_authored(&self, value: RuntimeViewModelPointer) {
        let mut state = self.state.borrow_mut();
        state.value = value;
        state.linked_instance = None;
        state.cell.notify_bindings_value_changed();
    }

    fn linked_instance(&self) -> Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        self.state.borrow().linked_instance.as_ref().map(Rc::clone)
    }

    fn set_linked_instance_silent(
        &self,
        linked_instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    ) {
        self.state.borrow_mut().linked_instance = linked_instance;
    }

    fn link_instance(&self, linked_instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>) {
        let mut state = self.state.borrow_mut();
        state.linked_instance = Some(linked_instance);
        // The C++ retained pointer setter dirties unconditionally, including
        // same-pointer reassignment. The compatibility pointer projection can
        // remain equal, so emit the property dirt explicitly.
        state.cell.notify_bindings_value_changed();
    }
}

/// The actual retained structural property behind an owned data-bind source.
///
/// C++ `DataBindContextValueList` and `DataBindContextValueViewModel` retain
/// their source property and synchronize from that object on dirt. This enum
/// is the Rust ownership-equivalent; the cell is only its dependency/dirt
/// identity and never stores a copied list or child projection.
#[derive(Debug, Clone)]
pub(crate) enum RuntimeOwnedViewModelStructuralSource {
    List(RuntimeOwnedViewModelListHandle),
    ViewModel(RuntimeOwnedViewModelEndpointSource),
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelEndpointSource {
    state: Rc<RefCell<RuntimeOwnedViewModelEndpointState>>,
}

impl RuntimeOwnedViewModelStructuralSource {
    pub(crate) fn cell(&self) -> RuntimeViewModelCell {
        match self {
            Self::List(list) => list.cell.clone(),
            Self::ViewModel(source) => source.state.borrow().cell.clone(),
        }
    }

    pub(crate) fn list_item_count(&self) -> Option<usize> {
        match self {
            Self::List(list) => Some(list.value.borrow().item_count),
            Self::ViewModel(_) => None,
        }
    }

    pub(crate) fn view_model_pointer(&self) -> Option<RuntimeViewModelPointer> {
        match self {
            Self::ViewModel(source) => {
                let state = source.state.borrow();
                let value = state
                    .linked_instance
                    .as_ref()
                    .map(|instance| RuntimeViewModelPointer::Retained {
                        allocation_identity: instance.borrow().allocation_identity,
                    })
                    .unwrap_or(state.value);
                Some(value)
            }
            Self::List(_) => None,
        }
    }
}

impl Clone for RuntimeOwnedViewModelEndpoint {
    fn clone(&self) -> Self {
        let state = self.state.borrow();
        Self {
            state: Rc::new(RefCell::new(RuntimeOwnedViewModelEndpointState {
                value: state.value,
                linked_instance: state.linked_instance.as_ref().map(Rc::clone),
                cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::ViewModel),
            })),
        }
    }
}
