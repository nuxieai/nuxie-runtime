// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_string.cpp`.
// Retained string bytes, cell identity, clone, and authored/imported construction.

/// One retained C++ `ViewModelInstanceString`: the cell owns the complete
/// payload as well as changed state and dependents. Retained consumers clone
/// the cell handle, while `Clone` creates a fresh cell with copied bytes.
#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelString {
    property_index: usize,
    pub(super) cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelString {
    pub(super) fn new(property_index: usize, value: Vec<u8>) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::String(value.into())),
        }
    }

    pub(super) fn value(&self) -> Arc<[u8]> {
        match self.cell.value() {
            RuntimeViewModelCellValue::String(value) => value,
            _ => unreachable!("string slot must retain a string cell"),
        }
    }

    pub(super) fn set_value(&mut self, value: &[u8]) -> bool {
        if self.value().as_ref() == value {
            return false;
        }
        self.cell
            .set_value(RuntimeViewModelCellValue::String(Arc::from(value)))
    }
}

impl RuntimeOwnedViewModelString {
    /// Identity-sharing view of the same retained C++ value object.
    #[cfg(test)]
    pub(super) fn share(&self) -> Self {
        Self {
            property_index: self.property_index,
            cell: self.cell.clone(),
        }
    }
}

impl Clone for RuntimeOwnedViewModelString {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value().to_vec())
    }
}

fn runtime_owned_view_model_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelString> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyString")
                        .then_some(RuntimeOwnedViewModelString::new(property_index, Vec::new()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_strings_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelString> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_string_value_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelString::new(
                property_index,
                value.as_bytes().to_vec(),
            ))
        })
        .collect()
}

fn runtime_owned_view_model_imported_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelString>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_strings_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
