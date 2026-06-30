# Data Binding Graph Default SymbolListIndex Source Mutation Runtime Contract

## Purpose

Add graph-owned default source mutation for
`ViewModelInstanceSymbolListIndex.propertyValue`.

The data-binding graph already carries symbol-list-index values for converter
execution. This slice adds the matching raw default-context source-node mutation
path, parallel to the existing number, boolean, string, color, enum, asset,
artboard, trigger, and view-model source mutation slices.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding existing
  converter-backed targets.
- Public Rust mutation by state-machine data-bind index.
- A C++ probe flag that mutates the raw generated `propertyValue` setter
  through `DataContext::getViewModelProperty`.
- C++ probe coverage through an existing symbol-list-index-to-string converter
  transition-condition consumer.

## Out Of Scope

- Symbol/list bindable target types.
- Owned or imported external symbol-list-index contexts.
- Stable public source handles beyond the current data-bind index seam.
- Reverse target-to-source propagation.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Mutating a default symbol-list-index source updates graph-owned source state
  when the default context is bound.
- Existing symbol-list-index converter probes continue to pass.
- Existing default source mutation probes continue to pass.
