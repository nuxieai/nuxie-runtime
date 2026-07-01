# Owned ViewModel Nested ViewModel Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested view-model
pointer paths without admitting imported-intermediate traversal, relative
lookup, or post-bind pointer evaluation behavior.

Owned generated nested view-model pointer mutation already resolves a path such
as `child/middle/leaf` before binding an owned runtime view-model context. This
slice exposes the same generated-child path through
`RuntimeOwnedViewModelViewModelSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/middle/leaf` into
  `RuntimeOwnedViewModelViewModelSourceHandle`.
- Relinking the owned generated-child view-model pointer through that handle
  before binding the owned context to a state machine.
- C++ probe comparison against the existing owned generated view-model
  name-path command, which resolves the nested pointer path and replaces the
  selected child instance.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, and list behavior beyond the existing committed APIs.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities beyond the selected pointer source.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and relinking an owned generated nested
view-model pointer source by handle produces the same state-machine advance,
view-model binding, and component update reports as the C++ owned generated
view-model name-path binding path, no-op repeat relinks report unchanged,
root-name lookup remains separate from slash-path lookup, and missing nested
paths remain unresolved.
