# Imported ViewModel Enum Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to enum value-index
sources without introducing a general mutable object or enum-definition handle
API.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyEnum(name)` source and mutate its
`propertyValue`. Rust models the same runtime fact with an immutable handle
that stores the resolved imported context identity and data-bind source path.
Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` enum override path, so every state
machine later bound through that context observes the same enum value index.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyEnum*` name into
  `RuntimeImportedViewModelEnumSourceHandle`.
- Resolving a slash-separated nested enum property path into the same handle
  type, using the existing `ViewModelPropertyViewModel` path resolver.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through
  `ViewModelInstanceRuntime::propertyEnum(name)` and the existing enum binding
  report surface.

Out of scope:

- Handles for symbol-list-index, asset, artboard, trigger, list, and
  view-model sources.
- Public enum-definition handles, enum labels/keys, or raw `propertyValue`
  object indexes.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: root and nested imported enum source handles resolved
from one shared imported context can mutate that context, a root handle cannot
mutate a different imported instance context, and both state machines bound
through the mutated context report the same enum source value index as C++.
