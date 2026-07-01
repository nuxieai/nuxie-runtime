# Imported Nested View-Model Name-Path Runtime Contract

## Scope

Admit imported-context view-model pointer relinks addressed by a
slash-separated `ViewModelPropertyViewModel.name` path.

The supported Rust surface is:

```rust
RuntimeImportedViewModelInstanceContext::set_view_model_by_property_name_path(
    file,
    "child/grandchild",
    referenced_instance_index,
)
```

The C++ parity surface is:

```cpp
ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)
```

## Required Behavior

- Resolve each path segment through `ViewModelPropertyViewModel` metadata,
  starting at the imported context's root view model.
- Use the leaf view-model property's `viewModelReferenceId` to select the
  referenced view-model instance by `referenced_instance_index`.
- Store the selected imported instance as a
  `RuntimeViewModelPointer::Imported` override keyed by the resolved source
  path.
- Binding multiple state machines through the same
  `RuntimeImportedViewModelInstanceContext` must expose the same relinked
  view-model source.
- Return `false` for invalid paths, non-view-model leaves, missing referenced
  view models, out-of-range referenced instances, or no-op relinks.

## Out Of Scope

- Mutating `RuntimeFile` or imported `ViewModelInstance` storage in place.
- Public object-handle APIs for arbitrary imported instance mutation.
- Reverse target-to-source propagation.
- Relative, parent, or manifest-backed view-model paths beyond the explicit
  slash-separated property-name path.
- Listener-owned data binding, nested artboard propagation, and generalized
  runtime update queues.

## Evidence

- `state_machine_imported_viewmodel_nested_viewmodel_source_name_path_relink_is_shared_across_state_machines_matches_cpp_probe`
  covers two authored state machines bound through one imported context after
  relinking `child/grandchild` to referenced instance index `1`.
