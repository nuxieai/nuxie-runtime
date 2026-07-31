// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_asset_font_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceAssetFontRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceAssetFontRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::AssetFont,
                cell,
            ),
        }
    }

    pub fn set_value(&self, font_bytes: Option<Arc<[u8]>>) -> bool {
        self.value.cell().set_live_font_bytes(font_bytes)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
