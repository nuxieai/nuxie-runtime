# Data Binding Graph Owned ViewModel Imported-Intermediate Runtime Contract

## Purpose

Admit the first owned view-model pointer path whose intermediate segment is an
imported replacement rather than a generated owned child.

C++ `ViewModelInstanceRuntime::replaceViewModelByName("child", importedChild)`
stores the imported child as the owned root's `child`
`referenceViewModelInstance()`. Later data-context binding for an absolute
path such as `[root, child, grandchild]` traverses through that imported child
and reads its existing nested `ViewModelInstanceViewModel` reference.

This slice deliberately does not mutate that imported intermediate. The C++
probe pins that an attempted root-runtime
`replaceViewModel("child/grandchild", value)` does not change the source
observed by this state-machine data bind for the admitted fixture.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Absolute `DataBindContext.sourcePathIds` with one imported intermediate:
  `[rootViewModelId, childPropertyId, nestedPropertyId]`.
- Root `ViewModelPropertyViewModel` replacement by imported instance index.
- Reading the imported intermediate's existing nested
  `ViewModelInstanceViewModel` reference.
- C++ probe coverage showing the source and target use the imported child's
  existing nested reference.
- C++ probe coverage showing attempted nested relink through the imported
  intermediate is unsupported for this path and leaves the same source/target.

## Out Of Scope

- Persistent mutation of imported `RuntimeFile` instances across contexts.
- Public property-name handles or stable object handles.
- Imported intermediates deeper than one segment.
- Nested scalar, list, symbol, asset, artboard, or trigger mutation through
  imported intermediates.
- Reverse target-to-source propagation.
- Broader update queues, relative paths, parent paths, listener-owned data
  binding, and nested artboard propagation.

## Completion Checks

- `set_view_model_by_property_path(&[child], importedIndex)` makes
  `[child, grandchild]` resolve through the selected imported child instance.
- Binding reports the imported child's existing grandchild instance index.
- `set_view_model_by_property_path(&[child, grandchild], index)` returns false
  after `child` is an imported intermediate, matching the C++ unsupported
  boundary pinned by the probe.
- Existing root owned replacement, generated-only recursive relink, default
  relink, imported relink, target-to-source, and raw setter probes continue to
  pass.
