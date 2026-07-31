// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_string_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceStringRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceStringRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::String,
                cell,
            ),
        }
    }

    pub fn value(&self) -> Arc<[u8]> {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::String(value) => value,
            _ => unreachable!("string runtime must retain a string cell"),
        }
    }

    pub fn value_string(&self) -> Option<String> {
        String::from_utf8(self.value().to_vec()).ok()
    }

    pub fn set_value(&self, value: impl Into<Arc<[u8]>>) -> bool {
        self.value
            .cell()
            .set_value(RuntimeViewModelCellValue::String(value.into()))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
