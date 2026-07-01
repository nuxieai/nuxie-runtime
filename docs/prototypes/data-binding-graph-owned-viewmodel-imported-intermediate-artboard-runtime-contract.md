# Data Binding Graph Owned ViewModel Imported-Intermediate Artboard Runtime Contract

## Purpose

Admit the artboard sibling of the imported-intermediate source path for owned
view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/scene` from the imported child's existing
`ViewModelInstanceArtboard.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported artboard snapshots for
referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested artboard source paths whose final segment is a
  `ViewModelPropertyArtboard` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::Artboard`.
- A C++ probe comparison that observes the bound artboard id through an
  existing `TransitionViewModelCondition` that stays unsatisfied after the
  imported source updates the bindable away from the forced target value.

## Out Of Scope

- Mutating through imported intermediates.
- Number, boolean, string, color, enum, symbol-list-index, and asset reads
  already admitted; trigger, list, and view-model pointer source reads beyond
  the slices already admitted.
- Artboard instancing, rendering, nested-artboard propagation, cloned-artboard
  lifecycle semantics, imported-instance mutation sharing, stable public object
  handles, reverse propagation, broader update queues, relative/parent/name-
  based lookup, and listener-owned data binding.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  artboard data-bind source read the imported child's existing artboard id.
- A matching C++ probe and Rust report leave the same state-machine transition
  unsatisfied when that imported artboard source updates the bindable away from
  the forced comparator value.
- Existing owned generated nested artboard and imported-intermediate number,
  boolean, string, color, enum, symbol-list-index, and asset probes continue to
  pass.
