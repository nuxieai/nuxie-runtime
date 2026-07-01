# Data Binding Graph Default ViewModel Relink Runtime Contract

## Purpose

Add the first live relink path for default-context
`ViewModelInstanceViewModel` sources. This is distinct from the raw generated
`propertyValue` setter: relinking updates the cached
`referenceViewModelInstance` pointer and enqueues the source-to-target bind for
`BindablePropertyViewModel.propertyValue`. This contract also covers same-path
observer propagation for ordinary direct `ToTarget` view-model binds.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyViewModel.propertyValue`.
- Default root view-model contexts bound with
  `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` that resolve to
  `ViewModelInstanceViewModel` sources.
- Public Rust relink by data-bind index and referenced instance index.
- Updating every bound default-context view-model source node that shares the
  selected data-bind source path.
- C++ probe mutation through
  `--runtime-relink-default-view-model-source-viewmodel`, which calls the
  cached-reference replacement path on the default root instance.
- C++ probe coverage through the existing view-model pointer transition
  condition and view-model binding reports, proving the source and bindable
  target pointers both observe the replacement after normal state-machine
  advancement.

## Out Of Scope

- The raw generated `propertyValue` setter behavior, which remains covered by
  `docs/prototypes/data-binding-graph-viewmodel-source-mutation-runtime-contract.md`.
- Imported external context relinking.
- Owned context relinking beyond the existing root-property replacement path.
- Nested view-model paths, public object handles, property-name lookup APIs, or
  replacing generated owned child identities.
- List bindables and list item propagation.
- Reverse target-to-source propagation.
- Two-way relink target timing on the intermediate explicit data-context
  report.
- Broader dirty/update queue parity, pending add/remove handling, re-entry
  protection, relative paths, parent paths, nested paths, and listener-owned
  data binding.

## Completion Checks

- Relinking a default `ViewModelInstanceViewModel` source to referenced
  instance index `1` changes same-path graph source pointers observed by Rust.
- The same relink updates same-path `BindablePropertyViewModel.propertyValue`
  targets by the next normal state-machine advance.
- The C++ probe reports the same source and target instance indexes for the
  relinked data bind and a same-path observer bind.
- The raw generated setter test continues to prove that `propertyValue` index
  writes do not relink the cached pointer.
