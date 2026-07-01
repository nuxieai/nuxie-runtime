# Data Binding Graph Owned ViewModel Deep Imported-Intermediate Runtime Contract

## Purpose

Admit read-only traversal through more than one imported intermediate in an
owned root view-model context.

The previous imported-intermediate slice covered `[child, grandchild]` after
the generated `child` pointer was replaced by an imported child instance. This
slice extends only the read side to a deeper path such as
`[child, middle, leaf]`, where the imported child already references an
imported middle instance and that middle already references an imported leaf.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Root `ViewModelPropertyViewModel` replacement by imported instance index.
- Absolute `DataBindContext.sourcePathIds` with two imported intermediate
  segments after the root property.
- Reading existing `ViewModelInstanceViewModel` references from imported
  intermediates.
- C++ probe coverage showing the source and target use the imported leaf
  instance already present in the imported chain.

## Out Of Scope

- Mutating through imported intermediates.
- Persistent mutation of imported `RuntimeFile` instances across contexts.
- Public property-name handles or stable object handles.
- Nested scalar, list, symbol, asset, artboard, or trigger mutation through
  imported intermediates.
- Reverse target-to-source propagation.
- Broader update queues, relative paths, parent paths, listener-owned data
  binding, and nested artboard propagation.

## Completion Checks

- `set_view_model_by_property_path(&[child], importedIndex)` lets a deeper
  `[child, middle, leaf]` source path resolve through the selected imported
  child and its existing imported descendants.
- Binding reports the imported leaf instance index with C++ parity.
- The existing generated-only deep relink, one-imported-intermediate read, and
  unsupported imported-intermediate mutation probes continue to pass.
