# Data Binding Graph Owned ViewModel Imported-Intermediate Number Runtime Contract

## Purpose

Admit the first scalar source read through an imported intermediate in an
owned view-model context.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/amount` from the imported child's existing `ViewModelInstanceNumber`.
For this Rust slice, `RuntimeOwnedViewModelInstance` records imported number
snapshots for referenced view-model instances so graph binding can read the
same source value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested number source paths whose final segment is a
  `ViewModelPropertyNumber` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::Number`.
- A C++ probe comparison using
  `--runtime-bind-owned-view-model-viewmodel-state-machine-context` followed by
  `advancedDataContext()`.

## Out Of Scope

- Mutating through imported intermediates.
- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model pointer source reads through imported intermediates.
- Imported-instance mutation sharing, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  number data-bind source read the imported child's existing number value.
- The matching C++ probe and Rust report the same number source and target
  value after binding and advancing the data context.
- Existing owned generated nested number and imported-intermediate view-model
  pointer probes continue to pass.
