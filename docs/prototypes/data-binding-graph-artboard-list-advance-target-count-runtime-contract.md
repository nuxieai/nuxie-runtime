# Data Binding Graph Artboard List Advance Target-Count Runtime Contract

## Purpose

Extend the artboard-owned list-consumer runtime boundary by one public C++
entrypoint: after binding the default artboard view-model context, a
zero-second `Artboard::advance(0.0f)` applies the bound list value to the exact
`ArtboardComponentList` target's list-count report.

This follows the C++ runtime boundary observed by the probe. A direct
`Artboard::updateDataBinds(true)` call after binding still reports an empty
target list for the synthetic fixtures; the public post-bind advance is the
smallest admitted behavior that changes the target count.

## In Scope

- Default root view-model context bound through
  `Artboard::bindViewModelInstance(...)` in C++ and
  `ArtboardInstance::bind_default_view_model_artboard_list_context(...)` in
  Rust.
- One post-bind zero-second artboard advance, exposed in the C++ probe as
  `--runtime-advance-artboard-after-bind` and in Rust as
  `ArtboardInstance::advance_artboard_data_binds()`.
- Exact artboard-owned `DataBindContext` records targeting
  `ArtboardComponentList`.
- Direct `ViewModelInstanceList` sources, where the target list count becomes
  the imported source list item count after the post-bind advance.
- Direct `DataConverterNumberToList` sources, where the target list count
  becomes the generated list count after the post-bind advance.
- Preservation of the immediate-bind behavior covered by
  `data-binding-graph-artboard-list-consumer-runtime-contract.md`.

## Out Of Scope

- Child artboard clone surfaces, child state-machine instances, child data
  contexts, item identity reuse, disposal, reset, and recorder application.
- Runtime map-rule selection for created child artboards.
- Layout-node synchronization, virtualization, scroll constraints, list item
  transforms, ordered draw-index sorting, hit testing, drawing, and renderer
  integration.
- Direct `Artboard::updateDataBinds(true)` parity beyond documenting that it is
  not the admitted target-count boundary for these fixtures.
- Relative, parent, nested, listener-owned, and name-based source paths.
- Target-to-source list behavior and generated-list reverse conversion.

## Completion Checks

- The direct list-source fixture continues to match C++ at the immediate bind
  boundary with an empty target list.
- The direct list-source fixture matches C++ after post-bind advance with the
  target list count equal to the source list count.
- The direct `DataConverterNumberToList` fixture continues to match C++ at the
  immediate bind boundary with an empty target list and reset flag.
- The direct `DataConverterNumberToList` fixture matches C++ after post-bind
  advance with the target list count equal to the generated count.
- The remaining audit continues to list real component-list item instancing,
  map-rule-driven child artboard creation, layout, and virtualization as later
  runtime slices.
