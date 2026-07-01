# Owned ViewModel Nested Trigger Source Handle Runtime Contract

Purpose: extend owned runtime source handles to generated nested trigger paths
without admitting trigger dispatch, listener-owned events, or the broader owned
nested handle family.

Owned generated nested trigger mutation already resolves a path such as
`child/fire` before binding an owned runtime view-model context. This slice
exposes the same generated-child path through
`RuntimeOwnedViewModelTriggerSourceHandle`, while preserving root handle
compatibility through `property_index()` and adding a path-shaped handle view
for nested callers.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Resolving a generated child path such as `child/fire` into
  `RuntimeOwnedViewModelTriggerSourceHandle`.
- Mutating owned generated-child raw trigger-count storage through that handle
  before binding the owned context to a state machine.
- C++ probe comparison against the existing owned trigger name-path command,
  which resolves the parent view model and mutates the child trigger value.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, and artboard
  behavior beyond the existing committed APIs, plus list or view-model owned
  source handles beyond the existing committed APIs.
- Trigger firing APIs, listener-owned dispatch, callback dispatch, or trigger
  reset behavior beyond existing raw value comparison.
- Imported-intermediate owned paths, relative or parent lookup, and replacing
  generated child identities.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating an owned generated nested trigger
source by handle produces the same state-machine advance and component update
reports as the C++ owned trigger name-path binding path, no-op repeat writes
report unchanged, root-name lookup remains separate from slash-path lookup,
and missing nested paths remain unresolved.
