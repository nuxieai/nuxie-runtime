# Data Binding Graph Owned ViewModel Imported-Intermediate List Runtime Contract

## Purpose

Admit the list sibling of the imported-intermediate source path for owned
view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/items` from the imported child's existing `ViewModelInstanceList` item
count. For this Rust slice, `RuntimeOwnedViewModelInstance` records imported
list item-count snapshots for referenced view-model instances so graph binding
can read the same source length.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested list source paths whose final segment is a
  `ViewModelPropertyList` on the imported child.
- Source-to-target graph binding for `RuntimeDataBindGraphValue::List` item
  counts.
- A C++ probe comparison that observes the bound list length through existing
  state-machine list binding reports.

## Out Of Scope

- Mutating through imported intermediates.
- List item identity, item instance resolution, list target mutation,
  virtualization, component-list layout, map-rule selection, reverse
  propagation, broader update queues, imported-instance mutation sharing,
  stable public object handles, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested list
  data-bind source read the imported child's existing item count.
- A matching C++ probe and Rust report expose the same source list size and
  bindable-list target value for the imported source path.
- Existing owned generated nested list and imported-intermediate number,
  boolean, string, color, enum, symbol-list-index, asset, artboard, and trigger
  probes continue to pass.
