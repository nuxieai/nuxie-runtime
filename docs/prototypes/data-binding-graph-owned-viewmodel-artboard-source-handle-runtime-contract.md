# Owned ViewModel Artboard Source Handle Runtime Contract

Purpose: extend the owned runtime view-model source-handle family from asset
sources to artboard sources without admitting nested owned handles, nested
artboard instancing, or the remaining owned handle kinds.

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
  source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, nested artboard instancing,
  reverse target-to-source propagation, broader update queues, listener-owned
  data binding, and nested artboard propagation.

Completion condition: resolving and mutating a root owned artboard source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-artboard binding path, no-op repeat writes report
unchanged, and slash-path handle lookup remains unresolved.
