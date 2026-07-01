# Owned ViewModel Asset Source Handle Runtime Contract

Purpose: document the root owned asset source handle without changing root
lookup semantics or admitting the full owned nested handle family.

This slice resolves a root asset view-model property name on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelAssetSourceHandle`. Mutating through that handle writes
the same owned-context raw asset id storage used by `set_asset_by_property_index`
and `set_asset_by_property_name`, then ordinary owned-context binding refreshes
matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyAsset` / `ViewModelPropertyAssetImage` name lookup
  only.
- Public source-handle resolution for the root owned asset property index.
- Mutating owned asset id storage through that handle before binding the owned
  context to a state machine.
- C++ probe comparison against the existing owned-asset runtime context
  command.

Out of scope:

- Number, boolean, string, color, enum, and symbol-list-index behavior beyond
  the existing committed APIs, plus artboard, trigger, list, or view-model
  owned source-handle behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested asset paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned asset source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-asset binding path, no-op repeat writes report
unchanged, and root-name lookup stays separate from slash-path lookup.
