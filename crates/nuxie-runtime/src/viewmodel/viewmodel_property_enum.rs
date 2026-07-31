// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_property_enum.cpp`.
// Enum-property path typing plus the narrow metadata adapter.

pub(super) fn runtime_imported_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

pub(super) fn runtime_imported_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

pub(crate) fn runtime_default_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name_path(file, 0, property_path)
}
