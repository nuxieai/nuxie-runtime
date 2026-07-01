# Data Binding Graph Owned ViewModel Nested Relink Runtime Contract

## Purpose

Add the first nested owned view-model pointer relink path:
an owned root context can traverse one generated owned child
`ViewModelInstanceViewModel`, replace a nested child pointer with a referenced
imported instance, and propagate that pointer to
`BindablePropertyViewModel.propertyValue`.

This follows the root owned replacement and generated-child identity slices
while keeping owned traversal finite and import-file-backed.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Absolute `DataBindContext.sourcePathIds` with exactly one intermediate
  owned view-model property, for example `[rootViewModelId, childPropertyId,
  nestedViewModelPropertyId]`.
- Intermediate properties that resolve to generated owned child pointers.
- Final nested properties that are `ViewModelPropertyViewModel` values whose
  referenced imported view model exists.
- Public Rust mutation by owned property path and referenced instance index.
- C++ probe mutation through
  `--runtime-bind-owned-view-model-nested-viewmodel-state-machine-context`,
  which creates an owned root instance, calls
  `ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)`,
  and binds the owned root to the state machine.
- C++ probe coverage through existing view-model binding reports.

## Out Of Scope

- Deeper owned paths beyond one intermediate generated child.
- Nested scalar properties, list properties, and list item propagation.
- Traversal through an imported replacement as an intermediate path segment.
- Public property-name handles or object handles.
- Persistent mutation of imported `RuntimeFile` instances.
- Reverse target-to-source propagation.
- Broader update queues, relative paths, parent paths, listener-owned data
  binding, and nested artboard propagation.

## Completion Checks

- A nested owned source path resolves through the generated child while the
  owned context is bound.
- Relinking the nested `ViewModelInstanceViewModel` source to referenced
  instance index `1` changes the graph source pointer observed by Rust.
- Explicit data-context advance updates the
  `BindablePropertyViewModel.propertyValue` target.
- The C++ probe reports the same source and target instance indexes for the
  relinked nested owned data bind.
- Existing root owned replacement/generated-child, default relink, imported
  relink, and raw setter probes continue to pass.
