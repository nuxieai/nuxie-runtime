# Owned ViewModel Boolean Source Handle Runtime Contract

Purpose: document the root owned boolean source handle without changing root
lookup semantics or admitting the full owned nested handle family.

This slice resolves a root `ViewModelPropertyBoolean.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelBooleanSourceHandle`. Mutating through that handle writes
the same owned-context boolean storage used by `set_boolean_by_property_index`
and `set_boolean_by_property_name`, then ordinary owned-context binding
refreshes matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyBoolean` name lookup only.
- Public source-handle resolution for the root owned boolean property index.
- Mutating owned boolean storage through that handle before binding the owned
  context to a state machine.
- C++ probe comparison against the existing owned-boolean runtime context
  command.

Out of scope:

- Number behavior beyond the existing committed API, plus string, color, enum,
  symbol-list-index, asset, artboard, trigger, list, or view-model owned
  source-handle behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested boolean paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned boolean source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-boolean binding path, no-op repeat writes report
unchanged, and root-name lookup stays separate from slash-path lookup.
