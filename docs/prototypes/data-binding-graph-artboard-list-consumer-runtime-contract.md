# Data Binding Graph Artboard List Consumer Runtime Contract

## Purpose

Admit the first artboard-owned list-consumer data-binding path: a default
root view-model list value can be bound to an exact `ArtboardComponentList`
target, and a default root number value can be converted through
`DataConverterNumberToList` for the same target family.

This slice intentionally stops at the immediate artboard bind boundary. It
records the same source facts and target bookkeeping that C++ exposes after
`Artboard::bindViewModelInstance(...)`, before admitting list item instancing,
layout, or virtualization.

## In Scope

- Default root view-model context bound through
  `Artboard::bindViewModelInstance(...)` in the C++ probe and
  `ArtboardInstance::bind_default_view_model_artboard_list_context(...)` in
  Rust.
- Artboard-owned exact `DataBindContext` records whose target is exact
  `ArtboardComponentList`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct imported `ViewModelInstanceList` sources, reporting the resolved
  imported item count as `sourceListSize`.
- Direct `DataConverterNumberToList` over a default
  `ViewModelInstanceNumber` source, reporting the numeric source value as
  `sourceNumberValue`.
- C++ parity for immediate `ArtboardComponentList` target bookkeeping:
  `targetLocal`, `targetListSize`, and `targetShouldResetInstances`.
- C++ probe coverage for both direct list and number-to-list sources.

## Observed C++ Boundary

- Direct list source-to-list target binding resolves the source list size, but
  the target list remains empty and does not request reset at this boundary.
- `DataConverterNumberToList` resolves the number source and converter view
  model, but the target list also remains empty; C++ marks the target list as
  needing reset.
- Generated list item identities are proven separately at the binary/converter
  level. They are not instantiated into `ArtboardComponentList` here.

## Out Of Scope

- `ArtboardComponentList` item instancing, cloned child artboards, generated
  item identity propagation, and map-rule selection.
- List layout, virtualization, scrolling, or `updateComponents()` side effects.
- `BindablePropertyList` targets and list target-to-source propagation.
- Reverse conversion for `DataConverterNumberToList`.
- Owned runtime view-model list contexts.
- Relative-path, parent-path, nested-path, listener-owned, and nested-artboard
  data binding.
- Dirty queue, observer-list, pending add/remove, and re-entry scheduling
  beyond the immediate bind report pinned here.

## Completion Checks

- A default `ViewModelInstanceList` source bound to an
  `ArtboardComponentList` reports the same source size, target local, target
  list size, and reset flag as the C++ probe.
- A default `ViewModelInstanceNumber` source converted by
  `DataConverterNumberToList` reports the same source number, target local,
  target list size, and reset flag as the C++ probe.
- The runtime docs continue to list artboard list item instancing and layout as
  follow-up slices, not hidden work inside this source-to-target report.
