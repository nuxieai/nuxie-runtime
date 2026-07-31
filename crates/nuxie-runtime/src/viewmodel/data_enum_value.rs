// Direct Rust owner for pinned C++ `src/viewmodel/data_enum_value.cpp`.
// Import-stack ownership is validated by `nuxie-binary`; this adapter keeps
// authored enum-value lookup at the per-value owner boundary.

fn runtime_data_enum_value_index_for_instance(
    file: &RuntimeFile,
    source: &RuntimeObject,
) -> Option<usize> {
    file.view_model_instance_enum_value_index_for_object(source)
}
