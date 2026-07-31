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
                    (matches!(
                        property.type_name,
                        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom"
                    ) || runtime_view_model_property_is_system_enum(property))
                    .then_some(RuntimeOwnedViewModelEnum::new(property_index, 0))
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
            let value = runtime_data_enum_value_index_for_instance(file, source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelEnum::new(
                property_index,
                u64::try_from(value).ok()?,
            ))
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
