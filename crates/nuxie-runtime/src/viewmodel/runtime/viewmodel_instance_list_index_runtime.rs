// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_list_index_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceListIndexRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceListIndexRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::SymbolListIndex,
                cell,
            ),
        }
    }

    pub fn value(&self) -> u32 {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::SymbolListIndex(value) => value,
            _ => unreachable!("list-index runtime must retain a list-index cell"),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}

#[cfg(test)]
mod upstream_viewmodel_instance_list_index_runtime_tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use nuxie_schema::definition_by_name;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    fn list_index_runtime() -> (
        ViewModelInstanceRuntime,
        ViewModelInstanceListIndexRuntime,
    ) {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Row".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertySymbolListIndex",
                vec![property(
                    "ViewModelPropertySymbolListIndex",
                    "name",
                    AuthoringValue::String("index".to_owned()),
                )],
            ),
        ])
        .expect("list-index runtime fixture");
        let runtime = ViewModelRuntime::new(Rc::new(file), 0)
            .expect("ViewModel runtime")
            .create_instance()
            .expect("generated instance");
        let property = runtime
            .property_list_index("index")
            .expect("list-index property runtime");
        (runtime, property)
    }

    // Literal Rust port of
    // tests/unit_tests/runtime/viewmodel_instance_list_index_runtime_test.cpp:
    // "list index runtime reports type and reads value".
    #[test]
    fn list_index_runtime_reports_type_and_reads_value() {
        let (runtime, property) = list_index_runtime();
        assert_eq!(
            property.value_runtime().data_type(),
            ViewModelRuntimeDataType::SymbolListIndex
        );
        assert_eq!(property.value(), 0);

        assert!(
            runtime
                .handle()
                .borrow_mut()
                .set_symbol_list_index_by_property_name("index", 3)
        );
        assert_eq!(property.value(), 3);
        assert!(
            runtime
                .handle()
                .borrow_mut()
                .set_symbol_list_index_by_property_name("index", 7)
        );
        assert_eq!(property.value(), 7);
    }

    // Literal Rust port of the upstream changed/flush contract.
    #[test]
    fn list_index_runtime_reports_value_changes() {
        let (runtime, property) = list_index_runtime();
        assert!(!property.value_runtime().has_changed());
        assert!(!property.value_runtime().flush_changes());

        assert!(
            runtime
                .handle()
                .borrow_mut()
                .set_symbol_list_index_by_property_name("index", 1)
        );
        assert!(property.value_runtime().has_changed());
        assert!(property.value_runtime().flush_changes());
        assert!(!property.value_runtime().has_changed());
        assert!(!property.value_runtime().flush_changes());

        assert!(
            !runtime
                .handle()
                .borrow_mut()
                .set_symbol_list_index_by_property_name("index", 1)
        );
        assert!(!property.value_runtime().has_changed());

        assert!(
            runtime
                .handle()
                .borrow_mut()
                .set_symbol_list_index_by_property_name("index", 2)
        );
        assert!(property.value_runtime().has_changed());
    }
}
