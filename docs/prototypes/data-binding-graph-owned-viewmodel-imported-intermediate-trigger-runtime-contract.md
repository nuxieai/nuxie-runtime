# Data Binding Graph Owned ViewModel Imported-Intermediate Trigger Runtime Contract

## Purpose

Admit the trigger sibling of the imported-intermediate source path for owned
view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/fire` from the imported child's existing
`ViewModelInstanceTrigger.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported trigger-count snapshots for
referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested trigger source paths whose final segment is a
  `ViewModelPropertyTrigger` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::Trigger`.
- A C++ probe comparison that observes the bound trigger count through an
  existing `TransitionViewModelCondition` that stays unsatisfied after the
  imported source updates the bindable away from the authored target value.

## Out Of Scope

- Mutating through imported intermediates.
- Number, boolean, string, color, enum, symbol-list-index, asset, and artboard
  reads already admitted; list and view-model pointer source reads beyond the
  slices already admitted.
- Trigger firing semantics, listener dispatch, callback targets,
  imported-instance mutation sharing, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup, and
  nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  trigger data-bind source read the imported child's existing trigger count.
- A matching C++ probe and Rust report leave the same state-machine transition
  unsatisfied when that imported trigger source updates the bindable away from
  the authored comparator value.
- Existing owned generated nested trigger and imported-intermediate number,
  boolean, string, color, enum, symbol-list-index, asset, and artboard probes
  continue to pass.
