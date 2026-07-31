// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_property_enum_system.cpp`.
// System enums expose an empty shared enum catalog, matching pinned C++.

fn runtime_view_model_property_is_system_enum(property: &RuntimeObject) -> bool {
    property.type_name == "ViewModelPropertyEnumSystem"
}
