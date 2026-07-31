// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_artboard_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceArtboardRuntime {
    value: ViewModelInstanceValueRuntime,
    runtime_state: Rc<RefCell<RuntimeOwnedViewModelArtboardState>>,
}

impl ViewModelInstanceArtboardRuntime {
    fn new(
        name: impl Into<String>,
        cell: RuntimeViewModelCell,
        runtime_state: Rc<RefCell<RuntimeOwnedViewModelArtboardState>>,
    ) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Artboard,
                cell,
            ),
            runtime_state,
        }
    }

    pub fn set_value(&self, artboard: Option<RuntimeBindableArtboard>) -> bool {
        let same_artboard = match (&self.runtime_state.borrow().bindable_artboard, &artboard) {
            (Some(current), Some(next)) => current.ptr_eq(next),
            (None, None) => true,
            _ => false,
        };
        self.runtime_state
            .borrow_mut()
            .bound_view_model_instance
            .take();
        if same_artboard {
            return self
                .value
                .cell()
                .set_value(RuntimeViewModelCellValue::Artboard(u32::MAX));
        }
        self.runtime_state.borrow_mut().bindable_artboard = artboard;
        let changed = self
            .value
            .cell()
            .set_value(RuntimeViewModelCellValue::Artboard(u32::MAX));
        if !changed {
            self.value.cell().notify_bindings_value_changed();
        }
        true
    }

    pub fn set_view_model_instance(&self, instance: Option<ViewModelInstanceRuntime>) {
        self.runtime_state.borrow_mut().bound_view_model_instance =
            instance.map(|instance| instance.handle().clone());
    }

    pub fn artboard_name(&self) -> String {
        self.runtime_state
            .borrow()
            .bindable_artboard
            .as_ref()
            .map(|artboard| artboard.name().to_owned())
            .unwrap_or_default()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
