// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_enum_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceEnumRuntime {
    value: ViewModelInstanceValueRuntime,
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    property_index: usize,
}

impl ViewModelInstanceEnumRuntime {
    fn new(
        name: impl Into<String>,
        cell: RuntimeViewModelCell,
        file: Rc<RuntimeFile>,
        view_model_index: usize,
        property_index: usize,
    ) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Enum,
                cell,
            ),
            file,
            view_model_index,
            property_index,
        }
    }

    fn property(&self) -> Option<&RuntimeObject> {
        self.file
            .view_model(self.view_model_index)?
            .properties
            .get(self.property_index)
            .copied()
    }

    fn data_values(&self) -> Vec<&RuntimeObject> {
        self.property()
            .and_then(|property| {
                self.file
                    .data_enum_for_view_model_property_object(property)
            })
            .map(|data_enum| data_enum.values)
            .unwrap_or_default()
    }

    pub fn value(&self) -> String {
        let index = self.value_index() as usize;
        self.data_values()
            .get(index)
            .and_then(|value| value.string_property("key"))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn set_value(&self, value: &str) -> bool {
        let Some(index) = self
            .data_values()
            .iter()
            .position(|candidate| candidate.string_property("key") == Some(value))
        else {
            return false;
        };
        self.set_value_index(index as u32)
    }

    pub fn value_index(&self) -> u32 {
        let index = match self.value.cell().value() {
            RuntimeViewModelCellValue::Enum(value) => value,
            _ => unreachable!("enum runtime must retain an enum cell"),
        };
        if (index as usize) < self.data_values().len() {
            index
        } else {
            0
        }
    }

    pub fn set_value_index(&self, index: u32) -> bool {
        if (index as usize) >= self.data_values().len() {
            return false;
        }
        self.value
            .cell()
            .set_value(RuntimeViewModelCellValue::Enum(index))
    }

    pub fn values(&self) -> Vec<String> {
        self.data_values()
            .into_iter()
            .map(|value| value.string_property("key").unwrap_or_default().to_owned())
            .collect()
    }

    pub fn enum_type(&self) -> String {
        self.property()
            .and_then(|property| {
                self.file
                    .data_enum_for_view_model_property_object(property)
            })
            .and_then(|data_enum| data_enum.object.string_property("name"))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
