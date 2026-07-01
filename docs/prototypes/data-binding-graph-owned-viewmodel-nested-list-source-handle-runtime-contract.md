# Owned ViewModel Nested List Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested list paths
for item-count mutation only, without admitting list item identity, generated
list item instances, or item-level traversal.

Owned generated nested list mutation already resolves a path such as
`child/items` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelListSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/items` into
  `RuntimeOwnedViewModelListSourceHandle`.
- Mutating owned generated-child list item-count storage through that handle
  before binding the owned context to a state machine.
- C++ probe comparison against the existing owned list name-path command,
  which resolves the list path and adds blank items to set the observed size.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard, and
  trigger behavior beyond the existing committed APIs, plus view-model pointer
  owned source handles beyond the existing committed APIs.
- List item identity, cloned item instances, item-level view-model traversal,
  insertion/removal ordering, `instanceAt`, stable item handles, map-rule
  selection, layout, virtualization, scrolling, or rendering.
- `DataConverterNumberToList`, `DataConverterListToLength`, reverse
  conversion, and target-to-source list propagation for owned contexts.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating an owned generated nested list
source by handle produces the same state-machine advance and list binding
reports as the C++ owned list name-path binding path, no-op repeat writes
report unchanged, root-name lookup remains separate from slash-path lookup,
and missing nested paths remain unresolved.
