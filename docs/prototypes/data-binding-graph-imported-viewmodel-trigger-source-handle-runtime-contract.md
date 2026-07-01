# Imported ViewModel Trigger Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to root trigger
sources while preserving the documented unsupported nested trigger path
boundary and keeping listener/dispatch behavior out of scope.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyValue(name)` trigger source and mutate its
`propertyValue`. Rust models the same runtime fact with an immutable handle
that stores the resolved imported context identity and data-bind source path.
Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` trigger override path, so every state
machine later bound through that context follows the same admitted post-bind
advance behavior.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyTrigger.name` into
  `RuntimeImportedViewModelTriggerSourceHandle`.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through the root trigger by-name mutation path and the
  existing state-machine advance report surface. Trigger binding/source-count
  report parity is not admitted by this slice.

Out of scope:

- Slash-separated/nested trigger handle lookup. This remains governed by
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-trigger-name-path-unsupported-runtime-contract.md`.
- Handles for list and view-model sources.
- Listener notification, event dispatch, fire-action semantics, or broader
  trigger scheduling behavior.
- Public trigger object handles or raw `propertyValue` object indexes.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: a root imported trigger source handle resolved from one
shared imported context can mutate that context, cannot mutate a different
imported instance context, slash-path handle lookup stays unresolved, and a
state machine bound through the mutated context advances with the same trigger
source effect as C++.
