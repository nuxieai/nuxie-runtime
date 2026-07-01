# Owned ViewModel List Source Handle Runtime Contract

Purpose: document the root owned list source handle without changing root
lookup semantics or admitting list item identity, generated list item
instances, list item handles, or the full owned nested handle family.

This slice resolves a root `ViewModelPropertyList.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelListSourceHandle`. Mutating through that handle writes the
same owned-context list item-count storage used by
`set_list_item_count_by_property_index`, `set_list_item_count_by_property_name`,
and the single-segment `"items"` name path, then ordinary owned-context binding
refreshes matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyList` name lookup only.
- Public source-handle resolution for the root owned list property index.
- Mutating owned list item-count storage through that handle before binding the
  owned context to a state machine.
- C++ probe comparison against the existing owned-list runtime context command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard, and
  trigger behavior beyond the existing committed APIs, plus view-model pointer
  owned source-handle behavior beyond the existing committed APIs.
- List item identity, item-level view-model traversal, generated item
  instances, list item handles, map-rule selection, layout, or virtualization.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested list paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned list source by handle
produces the same state-machine advance and list binding reports as the
existing C++ owned-list binding path, no-op repeat writes report unchanged, and
root-name lookup stays separate from slash-path lookup.
