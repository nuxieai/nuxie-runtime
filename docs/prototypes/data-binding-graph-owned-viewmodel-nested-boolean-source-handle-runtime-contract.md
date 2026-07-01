# Owned ViewModel Nested Boolean Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested boolean paths
without admitting the broader owned nested handle family.

Owned generated nested boolean mutation already resolves a path such as
`child/enabled` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelBooleanSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/enabled` into
  `RuntimeOwnedViewModelBooleanSourceHandle`.
- Mutating owned generated-child boolean storage through that handle before
  binding the owned context to a state machine.
- C++ probe comparison against
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`.

Out of scope:

- Number behavior beyond the existing committed APIs, plus string, color,
  enum, symbol-list-index, asset, artboard, trigger, list, or view-model owned
  source handles beyond the existing committed APIs.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating an owned generated nested boolean
source by handle produces the same state-machine advance and component update
reports as the C++ owned boolean name-path binding path, no-op repeat writes
report unchanged, root-name lookup remains separate from slash-path lookup,
and missing nested paths remain unresolved.
