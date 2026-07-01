# Owned ViewModel Nested Enum Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested enum paths
without admitting the broader owned nested handle family.

Owned generated nested enum mutation already resolves a path such as
`child/choice` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelEnumSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/choice` into
  `RuntimeOwnedViewModelEnumSourceHandle`.
- Mutating owned generated-child enum value-index storage through that handle
  before binding the owned context to a state machine.
- C++ probe comparison against
  `ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`.

Out of scope:

- Number, boolean, string, and color behavior beyond the existing committed
  APIs, plus symbol-list-index, asset, artboard, trigger, list, or view-model
  owned source handles beyond the existing committed APIs.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating an owned generated nested enum
source by handle produces the same state-machine advance and component update
reports as the C++ owned enum name-path binding path, no-op repeat writes
report unchanged, root-name lookup remains separate from slash-path lookup,
and missing nested paths remain unresolved.
