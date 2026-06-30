# Data Binding Graph Owned ViewModel SymbolListIndex Context Runtime Contract

## Purpose

Add owned runtime `ViewModelInstanceSymbolListIndex` context binding to the
runtime data-binding graph.

Default and imported file-backed symbol-list-index contexts are already
probe-backed. This slice closes the matching owned-runtime context lane by
storing symbol-list-index values in `RuntimeOwnedViewModelInstance` and letting
`bind_owned_view_model_context` refresh graph source nodes from those values.

## In Scope

- Root `ViewModelPropertySymbolListIndex` properties on
  `RuntimeOwnedViewModelInstance`.
- Public Rust mutation by property index through
  `set_symbol_list_index_by_property_index`.
- `RuntimeDataBindGraphValue::SymbolListIndex` resolution from an owned
  context for root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- C++ probe coverage using `File::createViewModelInstance(...)`, raw
  `ViewModelInstanceSymbolListIndex.propertyValue` mutation, and
  `StateMachineInstance::bindViewModelInstance(...)`.
- Verification through an existing `DataConverterToString` transition-condition
  consumer.

## Out Of Scope

- Symbol/list bindable target types.
- Stable public source handles beyond property-index mutation.
- Default or imported external source mutation.
- Reverse target-to-source propagation.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Owned symbol-list-index values default to `0`.
- Mutating an owned symbol-list-index property and binding the owned context
  refreshes matching graph source nodes.
- The converted string target observes the owned value on the next explicit
  state-machine advance.
- Existing default and imported symbol-list-index probes continue to pass.
