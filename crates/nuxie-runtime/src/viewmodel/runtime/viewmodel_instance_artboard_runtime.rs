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
            let changed = self
                .value
                .cell()
                .set_value(RuntimeViewModelCellValue::Artboard(u32::MAX));
            if artboard.is_some() && !changed {
                // C++ `asset()` dirties bindings even when the retained
                // BindableArtboard pointer is unchanged.
                self.value.cell().notify_bindings_value_changed();
                return true;
            }
            return changed;
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

#[cfg(test)]
mod upstream_data_binding_artboard_tests {
    use super::*;
    use crate::ViewModelRuntime;
    use nuxie_binary::read_runtime_file;
    use std::path::PathBuf;

    fn artboard_property() -> (ViewModelInstanceRuntime, ViewModelInstanceArtboardRuntime) {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/data_binding_artboards_test.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = Rc::new(read_runtime_file(&bytes).expect("fixture parses"));
        let index = file
            .view_models()
            .iter()
            .position(|view_model| {
                view_model
                    .properties
                    .iter()
                    .any(|property| property.string_property("name") == Some("ab"))
            })
            .expect("fixture has view model with ab artboard property");
        let runtime = ViewModelRuntime::new(file, index)
            .expect("view-model runtime")
            .create_instance()
            .expect("view-model instance");
        let property = runtime.property_artboard("ab").expect("ab property");
        (runtime, property)
    }

    #[test]
    fn setting_a_bindable_artboard_clears_stale_bound_instance() {
        let (runtime, property) = artboard_property();
        let source_a = RuntimeBindableArtboard::new("ch1");
        let source_b = RuntimeBindableArtboard::new("ch2");

        assert!(property.set_value(Some(source_a)));
        property.set_view_model_instance(Some(runtime));
        assert!(
            property
                .runtime_state
                .borrow()
                .bound_view_model_instance
                .is_some()
        );

        assert!(property.set_value(Some(source_b)));
        assert!(
            property
                .runtime_state
                .borrow()
                .bound_view_model_instance
                .is_none()
        );
    }

    #[test]
    fn runtime_artboard_property_exposes_the_bound_artboard_name() {
        let (_runtime, property) = artboard_property();
        let source_a = RuntimeBindableArtboard::new("ch1");
        let source_b = RuntimeBindableArtboard::new("ch2");

        assert!(property.set_value(Some(source_a)));
        assert_eq!(property.artboard_name(), "ch1");
        assert!(property.set_value(Some(source_b)));
        assert_eq!(property.artboard_name(), "ch2");
        assert!(property.set_value(None));
        assert_eq!(property.artboard_name(), "");
    }
}
