// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_trigger_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceTriggerRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceTriggerRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Trigger,
                cell,
            ),
        }
    }

    pub fn trigger(&self) -> bool {
        self.value.cell().fire_trigger()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
