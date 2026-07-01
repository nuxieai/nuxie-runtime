# Data Binding Graph Owned ViewModel Imported-Intermediate Symbol-List-Index Name-Path Unsupported Runtime Contract

## Purpose

Pin the public symbol-list-index name-path mutation boundary for owned
view-model contexts after a generated child has been replaced with an imported
child instance.

C++ can bind through the imported child's existing
`ViewModelInstanceSymbolListIndex.propertyValue`, but an attempted public
`child/symbol` property mutation after replacing `child` with an imported
instance leaves the observed graph source unchanged. Rust keeps the same
boundary:
`RuntimeOwnedViewModelInstance::set_symbol_list_index_by_property_name_path`
returns `false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested symbol-list-index source paths such as `child/symbol`.
- The attempted C++ public path shape that resolves the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and writes the
  child's `ViewModelInstanceSymbolListIndex.propertyValue`.
- Verifying that both C++ and Rust preserve the imported child's existing
  symbol-list-index source value after binding and state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Asset, artboard, trigger, list, and view-model pointer name-path mutation
  boundaries.
- Stable public object handles, reverse propagation, broader update queues,
  relative/parent/name-based lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/symbol`, binds the owned context, and still behaves as if the imported
  child's original symbol-list-index value is selected.
- Rust rejects the same mutation through
  `set_symbol_list_index_by_property_name_path("child/symbol", value)` after
  `child` is imported.
- The state-machine advance reports stay equal between C++ and Rust.
