# Data Binding Graph Imported ViewModel Relink Runtime Contract

## Purpose

Add the first live relink path for imported view-model contexts:
`ViewModelInstanceViewModel` sources resolved from a bound imported context can
be replaced by referenced imported instance index, then propagated to
`BindablePropertyViewModel.propertyValue`.

This follows the default-context relink slice while keeping imported context
mutation scoped to the currently bound runtime graph source.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyViewModel.propertyValue`.
- Imported root view-model contexts bound with
  `bind_view_model_instance_context(file, view_model_index, instance_index)`.
- Root-only `DataBindContext.sourcePathIds` that resolve to
  `ViewModelInstanceViewModel` sources.
- Public Rust relink by data-bind index and referenced instance index, only
  while an imported context is currently bound.
- C++ probe mutation through
  `--runtime-relink-view-model-instance-source-viewmodel`, which calls the
  cached-reference replacement path on the selected imported root instance.
- C++ probe coverage through the existing view-model pointer transition
  condition and view-model binding reports.

## Out Of Scope

- Persistent mutation of `RuntimeFile` view-model instances across later
  context rebinds.
- Default-context relinking, covered by
  `docs/prototypes/data-binding-graph-default-viewmodel-relink-runtime-contract.md`.
- Owned context relinking beyond the existing root-property replacement path.
- Nested view-model paths, public object handles, property-name lookup APIs, or
  generated owned child identity replacement.
- List bindables and list item propagation.
- Reverse target-to-source propagation.
- Broader dirty/update queue parity, pending add/remove handling, re-entry
  protection, relative paths, parent paths, nested paths, and listener-owned
  data binding.

## Completion Checks

- Relinking an imported-context `ViewModelInstanceViewModel` source to
  referenced instance index `1` changes the graph source pointer observed by
  Rust.
- The same relink updates the `BindablePropertyViewModel.propertyValue` target
  on explicit data-context advance.
- The C++ probe reports the same source and target instance indexes for the
  relinked imported-context data bind.
- Existing default raw-setter, default relink, owned replacement, and owned
  generated-child pointer probes continue to pass.
