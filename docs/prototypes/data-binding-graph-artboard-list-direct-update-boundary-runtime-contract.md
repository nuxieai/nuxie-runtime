# Data Binding Graph Artboard List Direct-Update Boundary Runtime Contract

## Purpose

Pin the direct artboard `updateDataBinds(true)` boundary for exact
`ArtboardComponentList` list-consumer data binds.

After binding the default artboard view-model context, provenance-bound
pinned-C++ `d788e8ec` direct `Artboard::updateDataBinds(true)` applies the
bound list and updates the target list count. Rust must execute its
corresponding direct bind-container delegate before comparing that state;
context binding alone is the pre-update boundary.

## In Scope

- Default root view-model context bound through
  `Artboard::bindViewModelInstance(...)`.
- A direct post-bind `Artboard::updateDataBinds(true)` call, exposed in the C++
  probe as `--runtime-update-artboard-data-binds`.
- Exact artboard-owned `DataBindContext` records targeting
  `ArtboardComponentList`.
- Direct `ViewModelInstanceList` sources and direct
  `DataConverterNumberToList` sources.
- C++ parity for updating the immediate-bind target local, target-list size,
  source list/number facts, and reset flag.

## Out Of Scope

- The full post-bind artboard-advance behavior, covered by
  `data-binding-graph-artboard-list-advance-target-count-runtime-contract.md`.
- Child artboard instancing, item identity reuse/disposal, map-rule-driven
  child creation, layout, virtualization, rendering, and hit testing.
- General artboard data-bind scheduler parity, pending add/remove behavior,
  re-entry protection, and target-to-source list behavior.

## Completion Checks

- Direct list-source fixture matches C++ after bind plus direct
  `updateDataBinds(true)` with the applied target-list count.
- Direct `DataConverterNumberToList` fixture matches C++ after bind plus direct
  `updateDataBinds(true)` with the applied target-list count and preserved
  reset flag.
- The post-bind advance tests continue to prove the full artboard boundary
  reaches the same list-consumer state.
