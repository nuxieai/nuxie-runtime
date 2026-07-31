// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_number.cpp`.
// f32 value identity, exact equality guard, clone, and authored/imported construction.

// #RB-1: scalar VALUE storage is backed by retained `RuntimeViewModelCell`s
// (`view_model_cell.rs`). Every write lands in the exact cell so retained
// dependents observe it; no root mutation clock or generation poll remains.
// `Clone` on the slot structs is a DEEP copy (fresh cell, same value) and
// `RuntimeOwnedViewModelInstance::clone` is a whole-subtree deep copy that
// preserves internal sharing topology via one dedupe map per operation —
// live sharing stays exclusively via `RuntimeOwnedViewModelHandle`.

#[derive(Debug)]
struct RuntimeOwnedViewModelNumber {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelNumber {
    fn new(property_index: usize, value: f32) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(value)),
        }
    }

    fn value(&self) -> f32 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Number(value) => value,
            _ => unreachable!("owned number slot holds a non-number cell"),
        }
    }

    /// Same change contract as the replaced `value` field comparison: returns
    /// whether the stored value changed (NaN writes always report a change,
    /// exactly like the old `!=` guard).
    fn set_value(&mut self, value: f32) -> bool {
        if self.value() == value {
            return false;
        }
        self.cell
            .set_value(RuntimeViewModelCellValue::Number(value));
        true
    }
}

impl Clone for RuntimeOwnedViewModelNumber {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelNumber> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyNumber")
                        .then_some(RuntimeOwnedViewModelNumber::new(property_index, 0.0))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_numbers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelNumber> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_number_value_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelNumber::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_numbers_for_instance(
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
