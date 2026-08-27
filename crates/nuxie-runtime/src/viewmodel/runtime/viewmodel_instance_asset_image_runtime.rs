// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_asset_image_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceAssetImageRuntime {
    value: ViewModelInstanceValueRuntime,
    runtime_state: Rc<RefCell<RuntimeOwnedViewModelImageState>>,
}

impl ViewModelInstanceAssetImageRuntime {
    fn new(
        name: impl Into<String>,
        cell: RuntimeViewModelCell,
        runtime_state: Rc<RefCell<RuntimeOwnedViewModelImageState>>,
    ) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::AssetImage,
                cell,
            ),
            runtime_state,
        }
    }

    pub fn set_value(&self, image: Option<RuntimeViewModelImage>) -> bool {
        set_runtime_view_model_image(self.value.cell(), &self.runtime_state, image)
    }

    #[cfg(test)]
    fn value(&self) -> Option<RuntimeViewModelImage> {
        self.runtime_state.borrow().live_image.clone()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
