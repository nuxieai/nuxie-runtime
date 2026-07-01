# Data Binding Graph Owned ViewModel Imported-Intermediate Boolean Runtime Contract

## Purpose

Admit the boolean sibling of the imported-intermediate scalar source path for
owned view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/enabled` from the imported child's existing
`ViewModelInstanceBoolean.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported boolean snapshots for
referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested boolean source paths whose final segment is a
  `ViewModelPropertyBoolean` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::Boolean`.
- A C++ probe comparison that observes the bound boolean through an existing
  `TransitionViewModelCondition`.

## Out Of Scope

- Mutating through imported intermediates.
- Number, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model pointer source reads beyond the slices already admitted.
- Imported-instance mutation sharing, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  boolean data-bind source read the imported child's existing boolean value.
- A matching C++ probe and Rust report take the same state-machine transition
  when that imported boolean source satisfies the condition.
- Existing owned generated nested boolean and imported-intermediate number
  probes continue to pass.
