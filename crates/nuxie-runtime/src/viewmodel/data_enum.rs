// Direct Rust owner for pinned C++ `src/viewmodel/data_enum.cpp`.
// Authored enum ordering and instance-value construction stay file ordered.

fn runtime_owned_view_model_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelEnum> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if matches!(
                        property.type_name,
                        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom"
                    ) || runtime_view_model_property_is_system_enum(property)
                    {
                        RuntimeOwnedViewModelEnum::for_property(
                            file,
                            view_model_index,
                            property_index,
                        )
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_enums_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelEnum> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            RuntimeOwnedViewModelEnum::for_instance_value(file, view_model_index, source)
        })
        .collect()
}

fn runtime_owned_view_model_imported_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_enums_for_instance(
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
