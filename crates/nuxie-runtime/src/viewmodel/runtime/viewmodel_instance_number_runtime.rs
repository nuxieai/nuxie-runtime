// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_number_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceNumberRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceNumberRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Number,
                cell,
            ),
        }
    }

    pub fn value(&self) -> f32 {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::Number(value) => value,
            _ => unreachable!("number runtime must retain a number cell"),
        }
    }

    pub fn set_value(&self, value: f32) -> bool {
        self.value
            .cell()
            .set_value(RuntimeViewModelCellValue::Number(value))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
