# Data Binding Graph Owned ViewModel Imported-Intermediate Asset Runtime Contract

## Purpose

Admit the asset sibling of the imported-intermediate source path for owned
view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/image` from the imported child's existing
`ViewModelInstanceAssetImage.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported asset snapshots for
referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested asset source paths whose final segment is a
  `ViewModelPropertyAssetImage` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::Asset`.
- A C++ probe comparison that observes the bound asset through an existing
  `TransitionViewModelCondition`.

## Out Of Scope

- Mutating through imported intermediates.
- Number, boolean, string, color, enum, and symbol-list-index reads already
  admitted; artboard, trigger, list, and view-model pointer source reads beyond
  the slices already admitted.
- Stable public asset/object handles, asset decoding or rendering behavior,
  imported-instance mutation sharing, reverse propagation, broader update
  queues, relative/parent/name-based lookup, listener-owned data binding, and
  nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested asset
  data-bind source read the imported child's existing asset id.
- A matching C++ probe and Rust report take the same state-machine transition
  when that imported asset source satisfies the condition.
- Existing owned generated nested asset and imported-intermediate number,
  boolean, string, color, enum, and symbol-list-index probes continue to pass.
