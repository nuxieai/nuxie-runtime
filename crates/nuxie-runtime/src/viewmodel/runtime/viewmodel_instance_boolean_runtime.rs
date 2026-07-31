// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_boolean_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceBooleanRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceBooleanRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Boolean,
                cell,
            ),
        }
    }

    pub fn value(&self) -> bool {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::Boolean(value) => value,
            _ => unreachable!("boolean runtime must retain a boolean cell"),
        }
    }

    pub fn set_value(&self, value: bool) -> bool {
        self.value
            .cell()
            .set_value(RuntimeViewModelCellValue::Boolean(value))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
