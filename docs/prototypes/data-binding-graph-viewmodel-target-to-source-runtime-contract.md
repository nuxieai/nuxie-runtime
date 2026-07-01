# Data Binding Graph ViewModel Target-To-Source Runtime Contract

## Purpose

Extend graph-owned target-to-source runtime behavior to direct view-model
pointer binds.

This slice mirrors the C++ listener-style relink path for a
`BindablePropertyViewModel.propertyValue` target. A mutated bindable
view-model target writes a referenced `ViewModelInstance` pointer back into the
bound default `ViewModelInstanceViewModel` source, and other graph source nodes
with the same source path observe the relink before normal source-to-target
application.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceViewModel.propertyValue` sources whose referenced
  view-model instance IDs are imported and resolvable.
- Direct `BindablePropertyViewModel.propertyValue` targets.
- `ToSource | TwoWay` data-bind flags for the mutated target bind.
- Public Rust mutation by state-machine data-bind index and referenced
  instance index.
- Explicit `advance_data_context` draining dirty view-model target-to-source
  writes before normal source-to-target application.
- Same-path propagation to other graph source nodes so a second bind can
  observe the relink.
- C++ probe coverage through
  `--runtime-set-state-machine-bindable-viewmodel` and `viewModelBindings`
  reports, using the active data-context source path as the semantic source.

## Out Of Scope

- Pure `ToSource` without `TwoWay`.
- Public `updateDataBinds(true)` view-model pointer target-to-source behavior,
  covered by
  `docs/prototypes/data-binding-graph-viewmodel-public-update-observer-application-runtime-contract.md`.
- Reverse converter execution and converter groups in the target-to-source
  direction.
- Imported external contexts and owned runtime contexts for this reverse path.
- Public source-side relink APIs beyond the already documented raw generated
  setter behavior.
- List bindables, list item propagation, generated runtime lists, and
  number-to-list behavior.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this direct
  path.
- Post-relink transition evaluation as the oracle for this fixture: C++ can
  dereference a missing to-target data bind for a to-source view-model
  comparator, so the probe reports the source/target pointer state directly.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- Mutating the first two-way view-model target marks its graph binding
  target-to-source dirty.
- `advance_data_context` resolves the referenced imported view-model instance
  index and writes that pointer identity into the bound default
  `ViewModelInstanceViewModel` source.
- A second bind to the same source path observes the updated pointer identity
  after the same explicit data-context advance.
- C++ `viewModelBindings` reports and Rust graph accessors agree on the source
  and target referenced instance index.
- Existing direct target-to-source tests for number, boolean, string, color,
  enum, asset, artboard, symbol-list-index, and trigger still pass.
