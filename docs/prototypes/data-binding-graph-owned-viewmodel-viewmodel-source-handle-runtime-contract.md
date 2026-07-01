# Owned ViewModel ViewModel Source Handle Runtime Contract

Purpose: document the root owned view-model pointer source handle without
changing root lookup semantics or admitting imported-intermediate traversal or
the full owned nested handle family.

This slice resolves a root `ViewModelPropertyViewModel.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelViewModelSourceHandle`. Relinking through that handle
writes the same owned-context view-model pointer storage used by
`set_view_model_by_property_index` and the single-segment property-index path,
then ordinary owned-context binding refreshes matching graph source nodes
before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyViewModel` name lookup only.
- Public source-handle resolution for the root owned view-model pointer
  property index.
- Relinking the owned pointer source to one of the referenced view-model's
  authored instances before binding the owned context to a state machine.
- C++ probe comparison against the existing owned view-model pointer runtime
  context command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, and list behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Generated nested view-model pointer paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-viewmodel-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Public handles for imported intermediate children.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and relinking a root owned view-model pointer
source by handle produces the same state-machine advance and component update
reports as the existing C++ owned view-model pointer binding path, no-op repeat
relinks report unchanged, and root-name lookup stays separate from slash-path
lookup.
