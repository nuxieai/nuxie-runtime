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

    #[cfg(test)]
    fn testing_value(&self) -> Option<Arc<[u8]>> {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::AssetFont(value) => {
                value.live_font_bytes_arc().map(Arc::clone)
            }
            _ => unreachable!("asset-font runtime must retain an asset-font cell"),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
