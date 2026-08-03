// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_asset_blob_runtime.cpp`.

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
