# Imported ViewModel List Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to root list sources
without reopening the documented unsupported nested list path boundary.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyList(name)` list source and replace its item
count by clearing and appending list items. Rust models the same import-time
runtime fact with an immutable handle that stores the resolved imported context
identity and data-bind source path. Applying the handle writes through the
existing `RuntimeImportedViewModelInstanceContext` list item-count override
path, so every state machine later bound through that context observes the
same source list size.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyList.name` into
  `RuntimeImportedViewModelListSourceHandle`.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through the root list by-name mutation path and the
  existing bindable-list source-size report surface.

Out of scope:

- Slash-separated/nested list handle lookup. This remains governed by
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-list-name-path-unsupported-runtime-contract.md`.
- Stable list item handles, list item identity, list item value mutation, or
  virtualized/list layout behavior.
- Handles for view-model pointer sources.
- Public list object handles or raw `propertyList` object indexes.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: a root imported list source handle resolved from one
shared imported context can mutate that context's source item count, slash-path
handle lookup stays unresolved, repeated same-count writes report no change,
and a state machine bound through the mutated context reports the same
bindable-list source size as C++.
