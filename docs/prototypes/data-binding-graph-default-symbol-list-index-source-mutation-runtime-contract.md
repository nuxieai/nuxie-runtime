# Data Binding Graph Default SymbolListIndex Source Mutation Runtime Contract

## Purpose

Add graph-owned default source mutation for
`ViewModelInstanceSymbolListIndex.propertyValue`.

The data-binding graph already carries symbol-list-index values for converter
execution. This slice adds the matching raw default-context source-node mutation
path and same-path observer propagation, parallel to the existing number,
boolean, string, color, and enum source mutation observer slices.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding existing
  converter-backed targets and direct integer bindable targets.
- Public Rust mutation by state-machine data-bind index.
- Updating every bound default-context symbol-list-index source node that
  shares the selected data-bind source path, matching C++ mutation of the
  shared `ViewModelInstanceSymbolListIndex` value.
- A C++ probe flag that mutates the raw generated `propertyValue` setter
  through `DataContext::getViewModelProperty`.
- C++ probe coverage through an existing symbol-list-index-to-string converter
  transition-condition consumer plus a neighboring ordinary direct `ToTarget`
  integer bind that observes the same default source path.

## Out Of Scope

- Symbol/list bindable target types.
- Remaining default source mutation observer families beyond number, boolean,
  string, color, enum, and symbol-list-index.
- Owned or imported external symbol-list-index contexts.
- Stable public source handles beyond the current data-bind index seam.
- Reverse target-to-source propagation.
- Full dirty-list scheduler parity, pending add/remove behavior, and re-entry
  protection.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Mutating a default symbol-list-index source updates graph-owned source state
  when the default context is bound.
- A neighboring ordinary direct `ToTarget` bind with the same source path
  reports the updated source and applies the updated target on the next
  state-machine advance.
- Existing symbol-list-index converter probes continue to pass.
- Existing default source mutation probes continue to pass.
