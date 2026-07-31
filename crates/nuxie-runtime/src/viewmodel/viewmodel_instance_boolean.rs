// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_boolean.cpp`.
// Boolean value identity, mutation, clone, and authored/imported construction.

#[derive(Debug)]
struct RuntimeOwnedViewModelBoolean {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelBoolean {
    fn new(property_index: usize, value: bool) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Boolean(value)),
        }
    }

    fn value(&self) -> bool {
        match self.cell.value() {
            RuntimeViewModelCellValue::Boolean(value) => value,
            _ => unreachable!("owned boolean slot holds a non-boolean cell"),
        }
    }

    fn set_value(&mut self, value: bool) -> bool {
        self.cell
            .set_value(RuntimeViewModelCellValue::Boolean(value))
    }
}

impl Clone for RuntimeOwnedViewModelBoolean {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyBoolean")
                        .then_some(RuntimeOwnedViewModelBoolean::new(property_index, false))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_booleans_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_boolean_value_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelBoolean::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_booleans_for_instance(
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
