# Data Binding Graph ViewModel Source Mutation Runtime Contract

## Purpose

Add the first probe-backed raw mutation path for graph-owned view-model pointer
sources: default-context `ViewModelInstanceViewModel.propertyValue` indexes can
be written by referenced view-model instance index without relinking the cached
imported `referenceViewModelInstance`, matching the C++ generated
`propertyValue` setter behavior.

This slice closes the smallest public view-model pointer mutation path without
starting list binding, owned-context nested view-model values, or general
update-queue parity.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyViewModel.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceViewModel.propertyValue` sources whose referenced
  view-model instances are imported and resolvable.
- Public Rust mutation by data-bind index and referenced instance index, limited
  to the raw-index behavior C++ exposes through the generated setter.
- C++ probe mutation through
  `--runtime-set-default-view-model-source-viewmodel`.
- C++ probe coverage proving that raw-index mutation does not relink the cached
  pointer observed by a view-model pointer transition condition.

## Out Of Scope

- Public mutation by object handle or nested view-model path.
- Owned runtime view-model contexts carrying nested view-model pointer values.
- Imported external context view-model pointer mutation.
- Live view-model replacement/relink APIs such as C++ `updateViewModel` or
  `replaceViewModelByProperty`.
- List bindables and list item propagation.
- Reverse target-to-source propagation.
- Full dirty/update queue parity beyond this explicit data-context advance
  path.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Runtime graph view-model sources remember the imported instance IDs for their
  referenced view model.
- Public source mutation rejects non-view-model sources and out-of-range
  referenced instance indexes.
- Raw-index mutation leaves the cached imported pointer unchanged, so explicit
  data-context advance does not make a pointer equality transition observe a
  relinked instance.
- Existing root/null pointer and graph-owned view-model source probes continue
  to pass.
