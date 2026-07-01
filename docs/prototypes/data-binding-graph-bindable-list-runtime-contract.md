# Data Binding Graph Bindable List Runtime Contract

## Purpose

Add the first state-machine `BindablePropertyList.propertyValue` runtime
binding path. A default or imported view-model list source can bind to a
state-machine bindable list target, and `DataConverterNumberToList` can produce
the same list value family.

In C++, list values apply only to `DataBindListItemConsumer` targets, and
`BindablePropertyList` is not one of those consumers. This slice therefore
imports and reports the list source facts while preserving the C++ target
`propertyValue` behavior.

## In Scope

- State-machine-owned `DataBindContext` objects whose target is
  `BindablePropertyList.propertyValue`.
- Default and imported contexts already supported by the
  `RuntimeDataBindGraph`.
- Direct `ViewModelInstanceList` sources represented by source item count.
- `DataConverterNumberToList` over a number source represented by source number
  value and generated list value inside the graph.
- C++ probe reporting for list bindings through source facts and target scalar
  `propertyValue`.
- Rust public report helpers by data-bind index for source list size, source
  number value, and target `propertyValue`.

## Out Of Scope

- Applying list values to `BindablePropertyList.propertyValue`.
- List item identity, cloned item instances, list item view-model traversal, and
  generated child identity propagation.
- `ArtboardComponentList` item instancing, layout, virtualization, scrolling, or
  rendering.
- Target-to-source list propagation and reverse conversion for
  `DataConverterNumberToList`.
- Public list mutation APIs.
- Owned view-model list storage and owned-context list binding.
- Relative, parent, nested, listener-owned, and nested-artboard data binding.

## Completion Checks

- A direct `ViewModelInstanceList` source bound to
  `BindablePropertyList.propertyValue` reports the same source item count and
  target scalar as C++ after context binding and data-context advance.
- A `DataConverterNumberToList` source bound to
  `BindablePropertyList.propertyValue` reports the same source number and
  unchanged target scalar as C++.
- Existing artboard list-consumer, list-to-length, scalar bindable, and
  view-model pointer probes continue to pass.
