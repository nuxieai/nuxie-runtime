# Data Binding Graph Owned ViewModel Imported-Intermediate String Runtime Contract

## Purpose

Admit the string sibling of the imported-intermediate scalar source path for
owned view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/label` from the imported child's existing
`ViewModelInstanceString.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported string snapshots for
referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested string source paths whose final segment is a
  `ViewModelPropertyString` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::String`.
- A C++ probe comparison that observes the bound string through an existing
  `TransitionViewModelCondition`.

## Out Of Scope

- Mutating through imported intermediates.
- Number and boolean reads already admitted; color, enum, symbol-list-index,
  asset, artboard, trigger, list, and view-model pointer source reads beyond
  the slices already admitted.
- Imported-instance mutation sharing, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  string data-bind source read the imported child's existing string value.
- A matching C++ probe and Rust report take the same state-machine transition
  when that imported string source satisfies the condition.
- Existing owned generated nested string and imported-intermediate number and
  boolean probes continue to pass.
