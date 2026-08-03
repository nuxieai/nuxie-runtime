// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_value_runtime.cpp`.
// Shared typed-value identity and dependent-backed changed reporting.

#[derive(Debug)]
struct ViewModelInstanceValueRuntimeInner {
    name: String,
    data_type: ViewModelRuntimeDataType,
    cell: RuntimeViewModelCell,
    changed: RuntimeCellDirtSink,
}

impl Drop for ViewModelInstanceValueRuntimeInner {
    fn drop(&mut self) {
        self.cell.remove_dependent(&self.changed);
    }
}

/// Shared base of every typed runtime property wrapper.
///
/// Clones retain one exact wrapper allocation. The underlying property cell
/// retains the authored value independently, matching C++ where destroying a
/// runtime wrapper unregisters only that wrapper's dependent.
#[derive(Debug, Clone)]
pub struct ViewModelInstanceValueRuntime {
    inner: Rc<ViewModelInstanceValueRuntimeInner>,
}

impl ViewModelInstanceValueRuntime {
    fn new(
        name: impl Into<String>,
        data_type: ViewModelRuntimeDataType,
        cell: RuntimeViewModelCell,
    ) -> Self {
        let changed = RuntimeCellDirtSink::new();
        cell.add_dependent(&changed);
        Self {
            inner: Rc::new(ViewModelInstanceValueRuntimeInner {
                name: name.into(),
                data_type,
                cell,
                changed,
            }),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn data_type(&self) -> ViewModelRuntimeDataType {
        self.inner.data_type
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn has_changed(&self) -> bool {
        !self.inner.changed.peek_dirt().is_empty()
    }

    pub fn clear_changes(&self) {
        self.inner.changed.take_dirt();
    }

    pub fn flush_changes(&self) -> bool {
        !self.inner.changed.take_dirt().is_empty()
    }

    fn cell(&self) -> &RuntimeViewModelCell {
        &self.inner.cell
    }
}

#[derive(Debug, Clone)]
pub enum ViewModelInstanceRuntimeProperty {
    Number(ViewModelInstanceNumberRuntime),
    String(ViewModelInstanceStringRuntime),
    Boolean(ViewModelInstanceBooleanRuntime),
    Color(ViewModelInstanceColorRuntime),
    Enum(ViewModelInstanceEnumRuntime),
    Trigger(ViewModelInstanceTriggerRuntime),
    ListIndex(ViewModelInstanceListIndexRuntime),
    List(ViewModelInstanceListRuntime),
    AssetImage(ViewModelInstanceAssetImageRuntime),
    AssetFont(ViewModelInstanceAssetFontRuntime),
    AssetBlob(ViewModelInstanceAssetBlobRuntime),
    Artboard(ViewModelInstanceArtboardRuntime),
}

// Cycle S4 regroups upstream `viewmodel_instance_asset_blob_runtime.cpp`
// beside its shared value wrapper until pin advance enrolls a direct row.
#[derive(Debug, Clone)]
pub struct ViewModelInstanceAssetBlobRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceAssetBlobRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::AssetBlob,
                cell,
            ),
        }
    }

    pub fn set_value(&self, bytes: Option<Arc<[u8]>>) -> bool {
        self.value.cell().set_live_blob_bytes(bytes)
    }

    pub fn value(&self) -> Option<Arc<[u8]>> {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::AssetBlob(value) => value.live_blob_bytes_arc(),
            _ => None,
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}

impl ViewModelInstanceRuntimeProperty {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        match self {
            Self::Number(value) => value.value_runtime(),
            Self::String(value) => value.value_runtime(),
            Self::Boolean(value) => value.value_runtime(),
            Self::Color(value) => value.value_runtime(),
            Self::Enum(value) => value.value_runtime(),
            Self::Trigger(value) => value.value_runtime(),
            Self::ListIndex(value) => value.value_runtime(),
            Self::List(value) => value.value_runtime(),
            Self::AssetImage(value) => value.value_runtime(),
            Self::AssetFont(value) => value.value_runtime(),
            Self::AssetBlob(value) => value.value_runtime(),
            Self::Artboard(value) => value.value_runtime(),
        }
    }

    pub fn data_type(&self) -> ViewModelRuntimeDataType {
        self.value_runtime().data_type()
    }

    pub fn name(&self) -> &str {
        self.value_runtime().name()
    }
}
