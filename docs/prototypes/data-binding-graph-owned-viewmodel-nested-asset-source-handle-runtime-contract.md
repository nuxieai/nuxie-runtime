# Owned ViewModel Nested Asset Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested asset paths
without admitting the broader owned nested object handle family.

Owned generated nested asset mutation already resolves a path such as
`child/image` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelAssetSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/image` into
  `RuntimeOwnedViewModelAssetSourceHandle`.
- Mutating owned generated-child asset-id storage through that handle before
  binding the owned context to a state machine.
- C++ probe comparison against the existing owned asset name-path command,
  which resolves the parent view model and mutates the child asset value.

Out of scope:

- Number, boolean, string, color, enum, and symbol-list-index behavior beyond
  the existing committed APIs, plus artboard, trigger, list, or view-model
  owned source handles beyond the existing committed APIs.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating an owned generated nested asset
source by handle produces the same state-machine advance and component update
reports as the C++ owned asset name-path binding path, no-op repeat writes
report unchanged, root-name lookup remains separate from slash-path lookup,
and missing nested paths remain unresolved.
