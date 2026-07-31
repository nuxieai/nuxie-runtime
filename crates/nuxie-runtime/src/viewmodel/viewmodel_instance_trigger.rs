// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_trigger.cpp`.
// Trigger counters, change notification, clone, import, and advance reset.

#[derive(Debug)]
struct RuntimeOwnedViewModelTrigger {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelTrigger {
    fn new(property_index: usize, value: u64) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(value)),
        }
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Trigger(value) => value,
            _ => unreachable!("owned trigger slot holds a non-trigger cell"),
        }
    }

    fn set_value(&mut self, value: u64) -> bool {
        self.cell
            .set_value(RuntimeViewModelCellValue::Trigger(value))
    }

    fn advanced(&self) {
        self.cell.advanced();
    }
}

impl Clone for RuntimeOwnedViewModelTrigger {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyTrigger")
                        .then_some(RuntimeOwnedViewModelTrigger::new(property_index, 0))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_triggers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_trigger_count_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelTrigger::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_triggers_for_instance(
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
