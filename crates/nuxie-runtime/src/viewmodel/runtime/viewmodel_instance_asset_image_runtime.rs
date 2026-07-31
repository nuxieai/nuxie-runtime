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
        let same_image = match (&self.runtime_state.borrow().live_image, &image) {
            (Some(current), Some(next)) => current.ptr_eq(next),
            (None, None) => true,
            _ => false,
        };
        if same_image {
            return self
                .value
                .cell()
                .set_value(RuntimeViewModelCellValue::AssetImage(u32::MAX));
        }
        self.runtime_state.borrow_mut().live_image = image;
        let changed = self
            .value
            .cell()
            .set_value(RuntimeViewModelCellValue::AssetImage(u32::MAX));
        if !changed {
            self.value.cell().notify_bindings_value_changed();
        }
        true
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
