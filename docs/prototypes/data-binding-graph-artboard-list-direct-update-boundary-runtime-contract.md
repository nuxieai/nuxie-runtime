# Data Binding Graph Artboard List Direct-Update Boundary Runtime Contract

## Purpose

Pin the direct artboard `updateDataBinds(true)` boundary for exact
`ArtboardComponentList` list-consumer data binds.

After binding the default artboard view-model context, C++ direct
`Artboard::updateDataBinds(true)` keeps the target list count at the same empty
value observed immediately after `Artboard::bindViewModelInstance(...)`.
The target count changes only at the separate public post-bind
`Artboard::advance(0.0f)` boundary.

## In Scope

- Default root view-model context bound through
  `Artboard::bindViewModelInstance(...)`.
- A direct post-bind `Artboard::updateDataBinds(true)` call, exposed in the C++
  probe as `--runtime-update-artboard-data-binds`.
- Exact artboard-owned `DataBindContext` records targeting
  `ArtboardComponentList`.
- Direct `ViewModelInstanceList` sources and direct
  `DataConverterNumberToList` sources.
- C++ parity for preserving the immediate-bind target local, empty target-list
  size, source list/number facts, and reset flag.

## Out Of Scope

- The post-bind advance target-count behavior, covered by
  `data-binding-graph-artboard-list-advance-target-count-runtime-contract.md`.
- Child artboard instancing, item identity reuse/disposal, map-rule-driven
  child creation, layout, virtualization, rendering, and hit testing.
- General artboard data-bind scheduler parity, pending add/remove behavior,
  re-entry protection, and target-to-source list behavior.

## Completion Checks

- Direct list-source fixture matches C++ after bind plus direct
  `updateDataBinds(true)` with an empty target list.
- Direct `DataConverterNumberToList` fixture matches C++ after bind plus direct
  `updateDataBinds(true)` with an empty target list and preserved reset flag.
- The post-bind advance tests continue to prove the target count changes only
  through the admitted advance boundary.
