# Data Binding Graph Bindable List Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for state-machine `BindablePropertyList.propertyValue`
targets when a list bind is target-to-source capable. This closes the current
list-target reverse-propagation question without admitting generated list item
runtime instances.

`BindablePropertyList` is not a `DataBindListItemConsumer` in C++. The runtime
therefore treats its `propertyValue` as a scalar target slot for reporting and
editing, while list-valued sources and `DataConverterNumberToList` generated
values do not get written back through this target path.

## In Scope

- State-machine-owned `DataBindContext` objects whose target is
  `BindablePropertyList.propertyValue`.
- Default root view-model contexts already supported by
  `RuntimeDataBindGraph`.
- Direct `ViewModelInstanceList` sources represented by imported source item
  count.
- `DataConverterNumberToList` over a default number source, represented by the
  source number and generated list value family already present in the graph.
- Mutating the bindable list target scalar `propertyValue` by data-bind index.
- C++ probe coverage for explicit `advancedDataContext()` and public
  `updateDataBinds(true)` target-to-source calls.
- Public `updateDataBinds(true)` coverage for `DataConverterNumberToList`; the
  explicit main-`ToSource | TwoWay` variant is covered by
  `docs/prototypes/data-binding-graph-number-to-list-main-to-source-target-to-source-runtime-contract.md`.

## Out Of Scope

- Writing edited list target values into `ViewModelInstanceList` sources.
- Generated-list reverse conversion for `DataConverterNumberToList` beyond the
  no-op target-to-source boundaries covered here and in
  `docs/prototypes/data-binding-graph-number-to-list-main-to-source-target-to-source-runtime-contract.md`.
- Generated list item identities, cloned item instances, and list item
  view-model traversal.
- `ArtboardComponentList` item instancing, map-rule selection, layout,
  virtualization, scrolling, or rendering.
- Owned view-model list storage and owned-context list binding.
- Relative, parent, nested, listener-owned, and nested-artboard data binding.

## Completion Checks

- A direct `ViewModelInstanceList` source bound to
  `BindablePropertyList.propertyValue` preserves the edited target scalar while
  reporting the same unchanged source list size as C++ after explicit
  `advancedDataContext()`.
- The same direct bind preserves the edited target scalar and unchanged source
  list size after public `updateDataBinds(true)`.
- A `DataConverterNumberToList` bind preserves the edited target scalar while
  reporting the same unchanged number source as C++ after public
  `updateDataBinds(true)`.
- A main-`ToSource | TwoWay` `DataConverterNumberToList` bind preserves the
  edited target scalar and unchanged number source in the companion explicit
  data-context contract.
- Existing list source-to-target, artboard list-consumer, and scalar
  target-to-source probes continue to pass.
