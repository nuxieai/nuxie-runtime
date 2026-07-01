# Owned ViewModel Symbol-List-Index Source Handle Runtime Contract

Purpose: document the root owned symbol-list-index source handle without
changing root lookup semantics or admitting the full owned nested handle
family.

This slice resolves a root `ViewModelPropertySymbolListIndex.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelSymbolListIndexSourceHandle`. Mutating through that
handle writes the same owned-context symbol-list-index storage used by
`set_symbol_list_index_by_property_index` and
`set_symbol_list_index_by_property_name`, then ordinary owned-context binding
refreshes matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertySymbolListIndex` name lookup only.
- Public source-handle resolution for the root owned symbol-list-index
  property index.
- Mutating owned symbol-list-index storage through that handle before binding
  the owned context to a state machine.
- C++ probe comparison against the existing owned-symbol-list-index runtime
  context command.

Out of scope:

- Number, boolean, string, color, and enum behavior beyond the existing
  committed APIs, plus asset, artboard, trigger, list, or view-model owned
  source-handle behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested symbol-list-index paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-symbol-list-index-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned symbol-list-index
source by handle produces the same state-machine advance and component update
reports as the existing C++ owned-symbol-list-index binding path, no-op repeat
writes report unchanged, and root-name lookup stays separate from slash-path
lookup.
