# Owned ViewModel Artboard Source Handle Runtime Contract

Purpose: document the root owned artboard source handle without changing root
lookup semantics or admitting the full owned nested handle family.

This slice resolves a root `ViewModelPropertyArtboard.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelArtboardSourceHandle`. Mutating through that handle
writes the same owned-context raw artboard id storage used by
`set_artboard_by_property_index` and `set_artboard_by_property_name`, then
ordinary owned-context binding refreshes matching graph source nodes before
state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyArtboard` name lookup only.
- Public source-handle resolution for the root owned artboard property index.
- Mutating owned artboard id storage through that handle before binding the
  owned context to a state machine.
- C++ probe comparison against the existing owned-artboard runtime context
  command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, and asset behavior
  beyond the existing committed APIs, plus trigger, list, or view-model owned
  source-handle behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested artboard paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, nested artboard instancing,
  reverse target-to-source propagation, broader update queues, listener-owned
  data binding, and nested artboard propagation.

Completion condition: resolving and mutating a root owned artboard source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-artboard binding path, no-op repeat writes report
unchanged, and root-name lookup stays separate from slash-path lookup.
