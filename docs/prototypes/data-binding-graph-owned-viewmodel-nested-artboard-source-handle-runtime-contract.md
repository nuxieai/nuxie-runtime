# Owned ViewModel Nested Artboard Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested artboard
paths without admitting nested artboard instancing or the broader owned nested
object handle family.

Owned generated nested artboard mutation already resolves a path such as
`child/scene` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelArtboardSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/scene` into
  `RuntimeOwnedViewModelArtboardSourceHandle`.
- Mutating owned generated-child raw artboard-id storage through that handle
  before binding the owned context to a state machine.
- C++ probe comparison against the existing owned artboard name-path command,
  which resolves the parent view model and mutates the child artboard value.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, and asset behavior
  beyond the existing committed APIs, plus trigger, list, or view-model owned
  source handles beyond the existing committed APIs.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Artboard referencer remapping, nested artboard instancing, render-side
  nested artboard behavior, and nested artboard propagation.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, and listener-owned data binding.

Completion condition: resolving and mutating an owned generated nested
artboard source by handle produces the same state-machine advance and
component update reports as the C++ owned artboard name-path binding path,
no-op repeat writes report unchanged, root-name lookup remains separate from
slash-path lookup, and missing nested paths remain unresolved.
