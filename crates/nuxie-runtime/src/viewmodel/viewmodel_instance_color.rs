// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_color.cpp`.
// Packed-color value identity, mutation, clone, and authored/imported construction.

#[derive(Debug)]
struct RuntimeOwnedViewModelColor {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelColor {
    fn new(property_index: usize, value: u32) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Color(value)),
        }
    }

    fn value(&self) -> u32 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Color(value) => value,
            _ => unreachable!("owned color slot holds a non-color cell"),
        }
    }

    fn set_value(&mut self, value: u32) -> bool {
        self.cell.set_value(RuntimeViewModelCellValue::Color(value))
    }
}

impl Clone for RuntimeOwnedViewModelColor {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelColor> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyColor")
                        .then_some(RuntimeOwnedViewModelColor::new(property_index, 0xFF000000))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_colors_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelColor> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_color_value_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelColor::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_colors_for_instance(
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
